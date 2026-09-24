use hidapi::{HidApi, MAX_REPORT_DESCRIPTOR_SIZE};
use serde::Serialize;
use std::{thread, time::Duration};

const X1_VENDOR_ID: u16 = 0x3151;
const X1_PRODUCT_ID: u16 = 0x5031;
const WIRED_X1_VENDOR_ID: u16 = 0x1d57;
const WIRED_X1_PRODUCT_IDS: [u16; 2] = [0xfa60, 0xfa65];
const WIRED_X1_PRODUCT_ID: u16 = 0x5032;
const HID_FEATURE_DATA_LEN: usize = 64;
const CONFIG_INTERFACE: i32 = 2;
const RAZER_VENDOR_ID: u16 = 0x1532;
const DEATHADDER_ESSENTIAL_PRODUCT_IDS: [u16; 3] = [0x006e, 0x0071, 0x0098];
const RAZER_MOUSE_INTERFACE: i32 = 0;
const RAZER_REPORT_LEN: usize = 91;
const RAZER_DPI_MIN: u16 = 100;
const RAZER_DPI_MAX: u16 = 6_400;

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

    set_x1_dpi(dpi)
}

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
}
