use hidapi::{HidApi, MAX_REPORT_DESCRIPTOR_SIZE};
use serde::Serialize;
use std::{
    ffi::{c_void, CStr},
    fs::OpenOptions,
    mem::zeroed,
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    ptr::null_mut,
    sync::Mutex,
    thread,
    time::Duration,
};

const X1_VENDOR_ID: u16 = 0x3151;
const X1_PRODUCT_ID: u16 = 0x5031;
const WIRED_X1_VENDOR_ID: u16 = 0x1d57;
const WIRED_X1_PRODUCT_IDS: [u16; 2] = [0xfa60, 0xfa65];
const WIRED_X1_PRODUCT_ID: u16 = 0x5032;
const DEXP_VENDOR_ID: u16 = 0x1d57;
const DEXP_GS_CRUSH_PRODUCT_IDS: [u16; 2] = [0xa001, 0xa011];
const DEXP_GS_CRUSH_WIRED_PRODUCT_ID: u16 = 0xa011;
const DEXP_DPI_MIN: u16 = 50;
const DEXP_DPI_MAX: u16 = 22_000;
const DEXP_DPI_REPORT_LEN: usize = 56;
const HID_FEATURE_DATA_LEN: usize = 64;
const CONFIG_INTERFACE: i32 = 2;
const RAZER_VENDOR_ID: u16 = 0x1532;
const DEATHADDER_ESSENTIAL_PRODUCT_IDS: [u16; 3] = [0x006e, 0x0071, 0x0098];
const RAZER_MOUSE_INTERFACE: i32 = 0;
const RAZER_REPORT_LEN: usize = 91;
const RAZER_DPI_MIN: u16 = 100;
const RAZER_DPI_MAX: u16 = 6_400;
static DEXP_WRITE_LOCK: Mutex<()> = Mutex::new(());
const FILE_SHARE_READ: u32 = 0x0000_0001;
const FILE_SHARE_WRITE: u32 = 0x0000_0002;
const HIDP_STATUS_SUCCESS: i32 = 0x0011_0000;

#[repr(C)]
#[derive(Default)]
struct HidpCaps {
    usage: u16,
    usage_page: u16,
    input_report_byte_length: u16,
    output_report_byte_length: u16,
    feature_report_byte_length: u16,
    reserved: [u16; 17],
    number_link_collection_nodes: u16,
    number_input_button_caps: u16,
    number_input_value_caps: u16,
    number_input_data_indices: u16,
    number_output_button_caps: u16,
    number_output_value_caps: u16,
    number_output_data_indices: u16,
    number_feature_button_caps: u16,
    number_feature_value_caps: u16,
    number_feature_data_indices: u16,
}

#[link(name = "hid")]
extern "system" {
    fn HidD_GetPreparsedData(
        hid_device_object: *mut c_void,
        preparsed_data: *mut *mut c_void,
    ) -> u8;
    fn HidD_FreePreparsedData(preparsed_data: *mut c_void) -> u8;
    fn HidP_GetCaps(preparsed_data: *const c_void, capabilities: *mut HidpCaps) -> i32;
    fn HidD_SetFeature(
        hid_device_object: *mut c_void,
        report_buffer: *const c_void,
        report_buffer_length: u32,
    ) -> u8;
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HidDiagnostic {
    vendor_id: String,
    product_id: String,
    interface_number: i32,
    usage_page: String,
    usage: String,
    manufacturer: Option<String>,
    product: Option<String>,
    report_descriptor: Option<String>,
    descriptor_error: Option<String>,
    feature_report: Option<String>,
    feature_report_error: Option<String>,
}

pub fn inspect_dpi_hardware() -> Result<Vec<HidDiagnostic>, String> {
    let api = HidApi::new().map_err(|error| format!("Could not initialise HID: {error}"))?;
    let diagnostics = api.device_list()
        .filter(|device| {
            let product = device.product_string().unwrap_or_default().to_ascii_lowercase();
            (device.vendor_id() == X1_VENDOR_ID
                    && (device.product_id() == X1_PRODUCT_ID
                        || device.product_id() == WIRED_X1_PRODUCT_ID))
                || (device.vendor_id() == WIRED_X1_VENDOR_ID
                    && WIRED_X1_PRODUCT_IDS.contains(&device.product_id()))
                || (device.vendor_id() == DEXP_VENDOR_ID
                    && DEXP_GS_CRUSH_PRODUCT_IDS.contains(&device.product_id()))
                || product.contains("mouse")
                || device.usage_page() == 0xffff
        })
        .map(|device| {
            let (report_descriptor, descriptor_error, feature_report, feature_report_error) = match device.open_device(&api) {
                Ok(handle) => {
                    let mut descriptor = [0_u8; MAX_REPORT_DESCRIPTOR_SIZE];
                    let (report_descriptor, descriptor_error) = match handle.get_report_descriptor(&mut descriptor) {
                        Ok(length) => (Some(hex(&descriptor[..length])), None),
                        Err(error) => (None, Some(format!("Could not read report descriptor: {error}"))),
                    };

                    // Interface 2 advertises one 64-byte Feature report without a report ID.
                    // hidapi requires a leading 0 byte for unnumbered reports, so the buffer is 65 bytes.
                    let mut feature = [0_u8; 65];
                    let (feature_report, feature_report_error) = match handle.get_feature_report(&mut feature) {
                        Ok(length) => (Some(hex(&feature[..length])), None),
                        Err(error) => (None, Some(format!("Could not read feature report: {error}"))),
                    };

                    (report_descriptor, descriptor_error, feature_report, feature_report_error)
                }
                Err(error) => (None, Some(format!("Could not open interface: {error}")), None, Some(format!("Could not open interface: {error}"))),
            };

            HidDiagnostic {
                vendor_id: format!("0x{:04x}", device.vendor_id()),
                product_id: format!("0x{:04x}", device.product_id()),
                interface_number: device.interface_number(),
                usage_page: format!("0x{:04x}", device.usage_page()),
                usage: format!("0x{:04x}", device.usage()),
                manufacturer: device.manufacturer_string().map(str::to_owned),
                product: device.product_string().map(str::to_owned),
                report_descriptor,
                descriptor_error,
                feature_report,
                feature_report_error,
            }
        })
        .collect::<Vec<_>>();

    if diagnostics.is_empty() {
        return Err("No mouse or vendor HID interfaces were found. Connect the mouse and try again.".into());
    }

    Ok(diagnostics)
}

pub fn read_x1_battery() -> Result<Option<u8>, String> {
    let api = HidApi::new().map_err(|error| format!("Could not initialize HID: {error}"))?;
    let path = api
        .device_list()
        .find(|device| {
            device.vendor_id() == X1_VENDOR_ID
                && (device.product_id() == X1_PRODUCT_ID
                    || device.product_id() == WIRED_X1_PRODUCT_ID)
                && device.interface_number() == CONFIG_INTERFACE
                && device.usage_page() == 0xffff
                && device.usage() == 0x0002
        })
        .map(|device| device.path().to_owned());

    let Some(path) = path else {
        return Ok(None);
    };

    let device = api
        .open_path(&path)
        .map_err(|error| format!("Could not open the X1 status interface: {error}"))?;
    let mut feature = [0_u8; HID_FEATURE_DATA_LEN + 1];
    let length = device
        .get_feature_report(&mut feature)
        .map_err(|error| format!("Could not read the X1 status report: {error}"))?;

    // On the X1's unnumbered status report, the observed fourth byte is the
    // battery percentage (for example, 0x5a reports 90%). Only surface values
    // that fit a normal percentage; any unknown report layout stays hidden.
    Ok((length > 3).then_some(feature[3]).filter(|value| (1..=100).contains(value)))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn get_dpi(vid: Option<&str>, pid: Option<&str>) -> Result<Option<u16>, String> {
    let (Some(vid), Some(pid)) = (parse_usb_id(vid), parse_usb_id(pid)) else {
        return Ok(None);
    };

    if vid == RAZER_VENDOR_ID && DEATHADDER_ESSENTIAL_PRODUCT_IDS.contains(&pid) {
        return read_deathadder_dpi(pid).map(Some);
    }

    Ok(None)
}

pub fn set_dpi(dpi: u16, vid: Option<&str>, pid: Option<&str>) -> Result<(), String> {
    let target = (parse_usb_id(vid), parse_usb_id(pid));
    if let (Some(RAZER_VENDOR_ID), Some(product_id)) = target {
        if DEATHADDER_ESSENTIAL_PRODUCT_IDS.contains(&product_id) {
            return set_deathadder_dpi(product_id, dpi);
        }
    }

    if let (Some(DEXP_VENDOR_ID), Some(product_id)) = target {
        if DEXP_GS_CRUSH_PRODUCT_IDS.contains(&product_id) {
            return set_dexp_gs_crush_dpi(product_id, dpi);
        }
    }

    set_x1_dpi(dpi)
}

fn set_dexp_gs_crush_dpi(product_id: u16, dpi: u16) -> Result<(), String> {
    if !(DEXP_DPI_MIN..=DEXP_DPI_MAX).contains(&dpi) {
        return Err(format!(
            "GS Crush DPI must be between {DEXP_DPI_MIN} and {DEXP_DPI_MAX}."
        ));
    }

    let _write_guard = DEXP_WRITE_LOCK
        .lock()
        .map_err(|_| "The GS Crush DPI writer could not be locked. Restart the app and try again.".to_string())?;
    let api = HidApi::new().map_err(|error| format!("Could not initialize HID: {error}"))?;
    let mut candidates = api
        .device_list()
        .filter(|device| {
            device.vendor_id() == DEXP_VENDOR_ID
                && device.product_id() == product_id
                && device.interface_number() == CONFIG_INTERFACE
        })
        .map(|device| {
            let path = device.path().to_owned();
            let windows_path = path.to_string_lossy().to_ascii_lowercase();
            // On Windows the firmware's writable feature-report collection is
            // COL04. Usage pages are not reported consistently across hidapi
            // backends, so use them only as the secondary preference.
            let score = if windows_path.contains("col04") {
                2
            } else if device.usage_page() == 0x000b {
                1
            } else {
                0
            };
            (score, path)
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| right.0.cmp(&left.0));
    if candidates.is_empty() {
        return Err("The GS Crush control interface was not found. Use USB or the 2.4 GHz receiver, then reconnect the mouse and try again.".to_string());
    }

    let report = build_dexp_dpi_report(dpi);
    let payload = if product_id == DEXP_GS_CRUSH_WIRED_PRODUCT_ID {
        &report[..52]
    } else {
        &report[..]
    };
    let candidate_count = candidates.len();
    let mut errors = Vec::new();
    for (_, path) in candidates {
        match send_padded_windows_feature_report(&path, payload) {
            Ok(report_length) => {
                // The Beken firmware needs time to persist a profile packet
                // before it will safely accept another command.
                thread::sleep(Duration::from_millis(250));
                eprintln!(
                    "GS Crush DPI profile sent as a {report_length}-byte Windows feature report"
                );
                return Ok(());
            }
            Err(error) => errors.push(error),
        }
    }

    Err(format!(
        "Could not send the DPI profile to any of the {candidate_count} GS Crush control collections: {}",
        errors.join("; ")
    ))
}

struct PreparsedData(*mut c_void);

impl Drop for PreparsedData {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: the pointer is returned by HidD_GetPreparsedData for the
            // lifetime of this guard and is released exactly once here.
            unsafe { HidD_FreePreparsedData(self.0) };
        }
    }
}

fn send_padded_windows_feature_report(path: &CStr, payload: &[u8]) -> Result<usize, String> {
    let path = path
        .to_str()
        .map_err(|_| "The GS Crush HID path is not valid UTF-8.".to_string())?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .open(path)
        .map_err(|error| format!("open: {error}"))?;
    let handle = file.as_raw_handle();
    let mut preparsed = null_mut();

    // SAFETY: handle is an open HID file handle and preparsed points to valid
    // storage for the pointer allocated by Windows.
    if unsafe { HidD_GetPreparsedData(handle, &mut preparsed) } == 0 {
        return Err(format!(
            "read feature-report capabilities: {}",
            std::io::Error::last_os_error()
        ));
    }
    let preparsed = PreparsedData(preparsed);
    // SAFETY: Windows initialized the preparsed data above, and caps is a
    // correctly laid-out HIDP_CAPS output buffer.
    let mut caps: HidpCaps = unsafe { zeroed() };
    let status = unsafe { HidP_GetCaps(preparsed.0, &mut caps) };
    if status != HIDP_STATUS_SUCCESS {
        return Err(format!("read HID capabilities: status 0x{status:08x}"));
    }

    let report_length = usize::from(caps.feature_report_byte_length);
    if report_length < payload.len() || report_length > 4096 {
        return Err(format!(
            "invalid Windows feature-report length {report_length} for a {}-byte DPI packet",
            payload.len()
        ));
    }
    let mut report = vec![0_u8; report_length];
    report[..payload.len()].copy_from_slice(payload);

    // DEXP's bundled hidapi performs this same full-length zero-padding before
    // HidD_SetFeature. Windows rejects the shorter protocol packet with error
    // 87 even though the mouse itself only consumes its first 56 bytes.
    // SAFETY: handle remains open and report is a valid buffer of the exact
    // FeatureReportByteLength returned by HidP_GetCaps.
    if unsafe {
        HidD_SetFeature(
            handle,
            report.as_ptr().cast(),
            report.len().try_into().expect("HID report length fits u32"),
        )
    } == 0
    {
        return Err(format!("write: {}", std::io::Error::last_os_error()));
    }

    Ok(report_length)
}

fn build_dexp_dpi_report(dpi: u16) -> [u8; DEXP_DPI_REPORT_LEN] {
    // GS Crush uses the same PAW3311/Beken profile format as the Attack Shark
    // X11. Preserve its standard stages and replace stage 1, then select it so
    // the requested DPI takes effect immediately.
    let mut report = [
        0x04, 0x38, 0x01, 0x00, 0x01, 0x3f, 0x20, 0x20,
        0x12, 0x25, 0x38, 0x4b, 0x75, 0x81, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
        0x01,
        0xff, 0x00, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0xff,
        0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0xff, 0x00, 0xff,
        0xff, 0x40, 0x00, 0xff, 0xff, 0xff,
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let (x, y, doubled) = encode_dexp_dpi(dpi);
    report[8] = x;
    report[16] = y;
    if doubled {
        report[6] |= 0x01;
    } else {
        report[6] &= !0x01;
    }
    report[7] &= !0x01;

    let checksum = report[3..=49]
        .iter()
        .fold(0_u16, |sum, byte| sum.wrapping_add(u16::from(*byte)))
        .to_be_bytes();
    report[50..52].copy_from_slice(&checksum);
    report
}

fn encode_dexp_dpi(dpi: u16) -> (u8, u8, bool) {
    if dpi == 20_100 {
        return (0xeb, 0x01, true);
    }

    let doubled = dpi > 10_000;
    let target = if doubled { (dpi + 1) / 2 } else { dpi };
    if target > 10_000 {
        let combined = 199 + ((target - 10_100) / 100);
        return (combined as u8, (combined >> 8) as u8, doubled);
    }

    let high_step = target > 5_000 && target % 100 == 0;
    let index = if high_step {
        (target / 100).saturating_sub(1) as usize
    } else {
        ((target.saturating_sub(50)) / 50) as usize
    };
    (
        DEXP_DPI_MAP.get(index).copied().unwrap_or(0xeb),
        u8::from(high_step),
        doubled,
    )
}

const DEXP_DPI_MAP: &[u8] = &[
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x15, 0x16,
    0x17, 0x18, 0x19, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x29, 0x2a, 0x2b, 0x2c,
    0x2d, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x43,
    0x44, 0x45, 0x46, 0x47, 0x48, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x51, 0x52, 0x53, 0x54, 0x55, 0x57, 0x58, 0x59,
    0x5a, 0x5b, 0x5c, 0x5e, 0x5f, 0x60, 0x61, 0x62, 0x63, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f,
    0x70, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x79, 0x7a, 0x7b, 0x7c, 0x7d, 0x7f, 0x80, 0x81, 0x82, 0x83, 0x84, 0x86,
    0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8d, 0x8e, 0x8f, 0x90, 0x91, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x9a, 0x9b, 0x9c,
    0x9d, 0x9e, 0x9f, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa7, 0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xae, 0xaf, 0xb0, 0xb1, 0xb2,
    0xb3, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf, 0xc0, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc9,
    0xca, 0xcb, 0xcc, 0xcd, 0xcf, 0xd0, 0xd1, 0xd2, 0xd3, 0xd4, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xdb, 0xdd, 0xde, 0xdf,
    0xe0, 0xe1, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xea, 0xeb,
];

fn set_x1_dpi(dpi: u16) -> Result<(), String> {
    if !(50..=40_000).contains(&dpi) {
        return Err("DPI must be between 50 and 40,000.".to_string());
    }

    let data = build_qmk_dpi_report(dpi);
    // The X1 uses an unnumbered 64-byte Feature report. hidapi requires a
    // leading report-ID byte, so the zero at index 0 represents “unnumbered”.
    let mut report = [0u8; HID_FEATURE_DATA_LEN + 1];
    report[1..].copy_from_slice(&data);

    let api = HidApi::new().map_err(|error| format!("Could not initialize HID: {error}"))?;
    let path = api
        .device_list()
        .find(|device| {
            device.vendor_id() == X1_VENDOR_ID
                && (device.product_id() == X1_PRODUCT_ID
                    || device.product_id() == WIRED_X1_PRODUCT_ID)
                && device.interface_number() == CONFIG_INTERFACE
                && device.usage_page() == 0xffff
                && device.usage() == 0x0002
        })
        .map(|device| device.path().to_owned())
        .ok_or_else(|| {
            "Mouse configuration interface was not found. Connect the X1 by USB-C or its receiver and try again.".to_string()
        })?;

    let device = api
        .open_path(&path)
        .map_err(|error| format!("Could not open the mouse configuration interface: {error}"))?;

    device
        .send_feature_report(&report)
        .map_err(|error| format!("Could not send the DPI configuration to the mouse: {error}"))?;

    Ok(())
}

fn parse_usb_id(value: Option<&str>) -> Option<u16> {
    let value = value?.trim();
    u16::from_str_radix(value.strip_prefix("0x").unwrap_or(value), 16).ok()
}

fn deathadder_handle(api: &HidApi, product_id: u16) -> Result<hidapi::HidDevice, String> {
    let preferred_path = api
        .device_list()
        .find(|device| {
            device.vendor_id() == RAZER_VENDOR_ID
                && device.product_id() == product_id
                && device.interface_number() == RAZER_MOUSE_INTERFACE
                && device.usage_page() == 0x0001
                && device.usage() == 0x0002
        })
        .or_else(|| {
            api.device_list().find(|device| {
                device.vendor_id() == RAZER_VENDOR_ID
                    && device.product_id() == product_id
                    && device.interface_number() == RAZER_MOUSE_INTERFACE
            })
        })
        .map(|device| device.path().to_owned())
        .ok_or_else(|| {
            "The DeathAdder control interface was not found. Reconnect the mouse and close Razer Synapse before trying again.".to_string()
        })?;

    api.open_path(&preferred_path)
        .map_err(|error| format!("Could not open the DeathAdder control interface: {error}. Close Razer Synapse and try again."))
}

fn set_deathadder_dpi(product_id: u16, dpi: u16) -> Result<(), String> {
    if !(RAZER_DPI_MIN..=RAZER_DPI_MAX).contains(&dpi) {
        return Err(format!(
            "DeathAdder DPI must be between {RAZER_DPI_MIN} and {RAZER_DPI_MAX}."
        ));
    }

    let api = HidApi::new().map_err(|error| format!("Could not initialize HID: {error}"))?;
    let device = deathadder_handle(&api, product_id)?;
    let report = build_razer_dpi_report(0x05, dpi);
    device
        .send_feature_report(&report)
        .map_err(|error| format!("Could not send the DPI command to the DeathAdder: {error}"))?;

    // OpenRGB uses the successful HID SetFeature result as completion for
    // write commands. A short pause prevents a following profile update from
    // overtaking the mouse firmware while it stores the new X/Y value.
    thread::sleep(Duration::from_millis(5));
    Ok(())
}

fn read_deathadder_dpi(product_id: u16) -> Result<u16, String> {
    let api = HidApi::new().map_err(|error| format!("Could not initialize HID: {error}"))?;
    let device = deathadder_handle(&api, product_id)?;
    let request = build_razer_dpi_report(0x85, 0);
    device
        .send_feature_report(&request)
        .map_err(|error| format!("Could not request the current DeathAdder DPI: {error}"))?;

    for _ in 0..5 {
        thread::sleep(Duration::from_millis(5));
        let mut response = [0_u8; RAZER_REPORT_LEN];
        match device.get_feature_report(&mut response) {
            Ok(length) if length >= 14
                && matches!(response[1], 0x01 | 0x02)
                && response[7] == 0x04
                && response[8] == 0x85 =>
            {
                let dpi = u16::from_be_bytes([response[10], response[11]]);
                if (RAZER_DPI_MIN..=RAZER_DPI_MAX).contains(&dpi) {
                    return Ok(dpi);
                }
            }
            _ => {}
        }
    }

    Err("The DeathAdder did not return a valid DPI response. Close Razer Synapse and try again.".to_string())
}

fn build_razer_dpi_report(command_id: u8, dpi: u16) -> [u8; RAZER_REPORT_LEN] {
    // Razer's standard 90-byte protocol report is prefixed with HID report ID
    // zero for hidapi on Windows. OpenRazer uses class 0x04, command 0x05 for
    // set DPI (0x85 for get DPI), seven argument bytes, and transaction 0xff
    // on all three DeathAdder Essential revisions.
    let mut report = [0_u8; RAZER_REPORT_LEN];
    report[0] = 0x00; // unnumbered HID feature report
    report[1] = 0x00; // request status
    report[2] = 0xff; // transaction ID
    report[6] = 0x07; // argument bytes
    report[7] = 0x04; // miscellaneous command class
    report[8] = command_id;

    if command_id == 0x05 {
        let dpi = dpi.to_be_bytes();
        report[9] = 0x01; // VARSTORE, matching OpenRazer's setter
        report[10..12].copy_from_slice(&dpi);
        report[12..14].copy_from_slice(&dpi);
    }

    report[89] = report[3..89].iter().fold(0_u8, |checksum, byte| checksum ^ byte);
    report
}

fn build_qmk_dpi_report(dpi: u16) -> [u8; HID_FEATURE_DATA_LEN] {
    // Captured from qmk.top’s working X1 SendFeatureReport request. The first
    // DPI stage is stored twice (X/Y) as a little-endian u16 at offsets 8 and 24.
    let mut report = [
        0x54, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0xa5,
        0xb0, 0x1d, 0x60, 0x09, 0x80, 0x0c, 0xe0, 0x15,
        0x40, 0x1f, 0x40, 0x9c, 0x00, 0x00, 0x00, 0x00,
        0xb0, 0x1d, 0x60, 0x09, 0x80, 0x0c, 0xe0, 0x15,
        0x40, 0x1f, 0x40, 0x9c, 0x00, 0x00, 0x00, 0x00,
        0xff, 0x00, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00,
        0xff, 0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0x80,
        0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    let dpi = dpi.to_le_bytes();
    report[8..10].copy_from_slice(&dpi);
    report[24..26].copy_from_slice(&dpi);
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_deathadder_set_dpi_report() {
        let report = build_razer_dpi_report(0x05, 800);
        assert_eq!(report.len(), 91);
        assert_eq!(&report[..10], &[0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0x07, 0x04, 0x05, 0x01]);
        assert_eq!(&report[10..14], &[0x03, 0x20, 0x03, 0x20]);
        assert_eq!(report[89], report[3..89].iter().fold(0_u8, |crc, byte| crc ^ byte));
        assert_eq!(report[90], 0x00);
    }

    #[test]
    fn parses_usb_ids() {
        assert_eq!(parse_usb_id(Some("0x0098")), Some(0x0098));
        assert_eq!(parse_usb_id(Some("006e")), Some(0x006e));
        assert_eq!(parse_usb_id(None), None);
    }

    #[test]
    fn builds_gs_crush_dpi_profile() {
        let report = build_dexp_dpi_report(800);
        assert_eq!(report.len(), 56);
        assert_eq!(&report[..8], &[0x04, 0x38, 0x01, 0x00, 0x01, 0x3f, 0x20, 0x20]);
        assert_eq!(report[8], 0x12);
        assert_eq!(report[16], 0x00);
        assert_eq!(report[24], 0x01);
        let expected = report[3..=49].iter().fold(0_u16, |sum, byte| sum + u16::from(*byte));
        assert_eq!(&report[50..52], &expected.to_be_bytes());
    }

    #[test]
    fn encodes_high_gs_crush_dpi() {
        let (x, y, doubled) = encode_dexp_dpi(22_000);
        assert_eq!((x, y, doubled), (0x81, 0x01, true));
    }
}
