#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

#[cfg(target_os = "windows")]
mod dpi;
#[cfg(target_os = "windows")]
mod remap;
#[cfg(target_os = "windows")]
mod rgb;

#[derive(Default)]
struct TrayState {
    minimize_to_tray: AtomicBool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DetectedDevice {
    id: String,
    name: String,
    manufacturer: Option<String>,
    vid: Option<String>,
    pid: Option<String>,
    connection: String,
    connected: bool,
    device_kind: String,
}

#[tauri::command]
fn detect_devices(show_hidden: bool) -> Result<Vec<DetectedDevice>, String> {
    #[cfg(target_os = "windows")]
    {
        let mut devices = windows_device_detection::detect_keyboards()?;
        let mice = windows_device_detection::detect_mice()?;
        devices.extend(mice.into_iter().filter(|mouse| show_hidden || is_relevant_mouse(mouse)));
        Ok(devices)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = show_hidden;
        Err("Device detection is currently available on Windows only.".to_string())
    }
}

#[tauri::command]
fn set_apex_rgb(colors: Vec<rgb::RgbColor>, brightness: u8) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    { rgb::set_apex_rgb(&colors, brightness) }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (colors, brightness);
        Err("Apex lighting control is currently available on Windows only.".to_string())
    }
}

#[tauri::command]
fn set_apex_rainbow(brightness: u8) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    { rgb::set_apex_rainbow(brightness) }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = brightness;
        Err("Apex lighting control is currently available on Windows only.".to_string())
    }
}

#[tauri::command]
fn inspect_dpi_hardware() -> Result<serde_json::Value, String> {
    #[cfg(target_os = "windows")]
    {
        serde_json::to_value(dpi::inspect_dpi_hardware()?)
            .map_err(|error| format!("Could not serialise HID diagnostics: {error}"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("DPI diagnostics are currently available on Windows only.".to_string())
    }
}

#[tauri::command]
fn get_x1_battery() -> Result<Option<u8>, String> {
    #[cfg(target_os = "windows")]
    {
        dpi::read_x1_battery()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(None)
    }
}

#[tauri::command]
fn get_dpi(vid: Option<String>, pid: Option<String>) -> Result<Option<u16>, String> {
    #[cfg(target_os = "windows")]
    {
        dpi::get_dpi(vid.as_deref(), pid.as_deref())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (vid, pid);
        Ok(None)
    }
}

#[tauri::command]
fn set_dpi(dpi: u16, vid: Option<String>, pid: Option<String>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    { dpi::set_dpi(dpi, vid.as_deref(), pid.as_deref()) }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (dpi, vid, pid);
        Err("DPI control is currently available on Windows only.".to_string())
    }
}

#[tauri::command]
fn set_minimize_to_tray(enabled: bool, state: tauri::State<'_, TrayState>) {
    state.minimize_to_tray.store(enabled, Ordering::Relaxed);
}

const STARTUP_REGISTRY_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const STARTUP_VALUE_NAME: &str = "UnnamedEnhancements";

#[tauri::command]
fn get_start_with_windows() -> bool {
    #[cfg(target_os = "windows")]
    {
        Command::new("reg.exe")
            .args(["query", STARTUP_REGISTRY_KEY, "/v", STARTUP_VALUE_NAME])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "windows"))]
    { false }
}

#[tauri::command]
fn set_start_with_windows(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let status = if enabled {
            let executable = std::env::current_exe()
                .map_err(|error| format!("Could not locate the app executable: {error}"))?;
            let command = format!("\"{}\" --startup", executable.display());
            Command::new("reg.exe")
                .args(["add", STARTUP_REGISTRY_KEY, "/v", STARTUP_VALUE_NAME, "/t", "REG_SZ", "/d"])
                .arg(command)
                .args(["/f"])
                .status()
        } else {
            Command::new("reg.exe")
                .args(["delete", STARTUP_REGISTRY_KEY, "/v", STARTUP_VALUE_NAME, "/f"])
                .status()
        }
        .map_err(|error| format!("Could not update Windows startup: {error}"))?;
        if status.success() || (!enabled && !get_start_with_windows()) {
            Ok(())
        } else {
            Err("Windows could not update the startup setting.".to_string())
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = enabled;
        Err("Start with Windows is available on Windows only.".to_string())
    }
}

#[tauri::command]
fn test_button_action(action: String, target: Option<String>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        remap::run_action(&action, target.as_deref())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (action, target);
        Err("Button actions are currently available on Windows only.".to_string())
    }
}

#[tauri::command]
fn apply_button_mappings(mappings: std::collections::HashMap<String, remap::ButtonBinding>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        remap::set_mappings(mappings);
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = mappings;
        Err("Button remapping is currently available on Windows only.".to_string())
    }
}
fn is_relevant_mouse(mouse: &DetectedDevice) -> bool {
    const GAMING_BRANDS: &[&str] = &[
        "attack shark", "logitech", "razer", "steelseries", "corsair", "glorious",
        "pulsar", "endgame gear", "zowie", "benq", "finalmouse", "lamzu", "darmoshark",
        "vxe", "redragon", "hyperx", "roccat", "asus", "cooler master",
    ];
    const HIDDEN_DEVICE_TERMS: &[&str] = &[
        "hid-compliant mouse", "microsoft input device", "touchpad", "trackpad",
        "remote desktop", "virtual", "vmware", "vbox",
    ];
    let text = format!("{} {}", mouse.name, mouse.manufacturer.as_deref().unwrap_or_default()).to_ascii_lowercase();
    if HIDDEN_DEVICE_TERMS.iter().any(|term| text.contains(term)) { return false; }
    if GAMING_BRANDS.iter().any(|brand| text.contains(brand)) { return true; }
    mouse.vid.is_some() && mouse.manufacturer.is_some() && !text.contains("microsoft") && !text.contains("unknown")
}

fn is_g305_family_name(mouse: &DetectedDevice) -> bool {
    let identity = format!(
        "{} {} {}",
        mouse.name,
        mouse.manufacturer.as_deref().unwrap_or_default(),
        mouse.id,
    )
    .to_ascii_lowercase();
    identity.contains("g305") || identity.contains("g304")
}

fn is_hyperx_haste_2_wireless_name(mouse: &DetectedDevice) -> bool {
    let identity = format!(
        "{} {} {}",
        mouse.name,
        mouse.manufacturer.as_deref().unwrap_or_default(),
        mouse.id,
    )
    .to_ascii_lowercase();
    identity.contains("pulsefire haste 2 wireless")
}

fn apply_known_mouse_identity(mouse: &mut DetectedDevice) {
    if mouse.vid.as_deref() == Some("0x3151")
        && matches!(mouse.pid.as_deref(), Some("0x5031") | Some("0x5032"))
    {
        mouse.name = "Attack Shark X1".to_string();
        mouse.manufacturer = Some("Attack Shark".to_string());
    } else if mouse.vid.as_deref() == Some("0x1d57")
        && matches!(mouse.pid.as_deref(), Some("0xa001") | Some("0xa011"))
    {
        // IDs recovered from DEXP's official GS Crush configuration utility.
        // A001 is the 2.4 GHz USB receiver and A011 is the wired USB-C mode.
        mouse.name = "DEXP GS Crush".to_string();
        mouse.manufacturer = Some("DEXP".to_string());
        mouse.connection = if mouse.pid.as_deref() == Some("0xa001") {
            "2.4 GHz receiver".to_string()
        } else {
            "USB-C (wired)".to_string()
        };
    } else if mouse.vid.as_deref() == Some("0x1bf8")
        && mouse.pid.as_deref() == Some("0x0f99")
    {
        // Verified from the USB device descriptor of the wired white model.
        // A USBPcap trace showed only standard mouse input reports on endpoint
        // 0x81 and no host-visible report when its physical DPI button changed
        // the built-in 1200/1600/2400/3200 presets, so keep this detection-only.
        mouse.name = "SmartBuy RUSH Avatar SBM-724G-W".to_string();
        mouse.manufacturer = Some("SmartBuy".to_string());
        mouse.connection = "Wired USB".to_string();
    } else if ((mouse.vid.as_deref() == Some("0x3554")
        && mouse.pid.as_deref() == Some("0xfa09"))
        || (mouse.vid.as_deref() == Some("0x03f0")
            && mouse.pid.as_deref() == Some("0x0f98")))
        || is_hyperx_haste_2_wireless_name(mouse)
    {
        // Current retail receivers can expose a generic 3554:FA09 HID identity;
        // older firmware has also been reported under HP's 03F0:0F98 identity.
        // Keep support detection-only until vendor feature reports are verified.
        mouse.name = "HyperX Pulsefire Haste 2 Wireless".to_string();
        mouse.manufacturer = Some("HyperX".to_string());
        if matches!(mouse.vid.as_deref(), Some("0x3554") | Some("0x03f0")) {
            mouse.connection = "USB-C / 2.4 GHz receiver".to_string();
        }
    } else if mouse.vid.as_deref() == Some("0x1532")
        && matches!(
            mouse.pid.as_deref(),
            Some("0x006e") | Some("0x0071") | Some("0x0098")
        )
    {
        // Original, white-edition, and 2021 DeathAdder Essential revisions.
        mouse.name = "Razer DeathAdder Essential".to_string();
        mouse.manufacturer = Some("Razer".to_string());
        mouse.connection = "Wired USB".to_string();
    } else if (mouse.vid.as_deref() == Some("0x046d")
        && matches!(mouse.pid.as_deref(), Some("0xc53f") | Some("0x4074")))
        || is_g305_family_name(mouse)
    {
        // G305 colour and special-edition models use the same LIGHTSPEED
        // family. G304 is the regional name for the same mouse. Windows may
        // expose either model name through the paired receiver.
        mouse.name = "Logitech G305 LIGHTSPEED".to_string();
        mouse.manufacturer = Some("Logitech G".to_string());
        mouse.connection = "LIGHTSPEED wireless".to_string();
    } else if mouse.vid.as_deref() == Some("0x258a")
        && matches!(mouse.pid.as_deref(), Some("0x0036") | Some("0x0027"))
    {
        // Original wired Model O / O- firmware revisions use Sinowealth's
        // vendor ID; recognise the family without sending any untested reports.
        mouse.name = "Glorious Model O Wired".to_string();
        mouse.manufacturer = Some("Glorious".to_string());
        mouse.connection = "Wired USB".to_string();
    }
}

#[cfg(test)]
mod device_identity_tests {
    use super::{apply_known_mouse_identity, is_relevant_mouse, DetectedDevice};

    #[test]
    fn recognises_smartbuy_rush_avatar() {
        let mut mouse = DetectedDevice {
            id: "HID\\VID_1BF8&PID_0F99".to_string(),
            name: "HID-compliant mouse".to_string(),
            manufacturer: Some("Microsoft".to_string()),
            vid: Some("0x1bf8".to_string()),
            pid: Some("0x0f99".to_string()),
            connection: "USB".to_string(),
            connected: true,
            device_kind: "mouse".to_string(),
        };

        apply_known_mouse_identity(&mut mouse);

        assert_eq!(mouse.name, "SmartBuy RUSH Avatar SBM-724G-W");
        assert_eq!(mouse.manufacturer.as_deref(), Some("SmartBuy"));
        assert_eq!(mouse.connection, "Wired USB");
        assert!(is_relevant_mouse(&mouse));
    }

    #[test]
    fn recognises_dexp_gs_crush_wired_and_receiver_ids() {
        for (pid, connection) in [
            ("0xa001", "2.4 GHz receiver"),
            ("0xa011", "USB-C (wired)"),
        ] {
            let mut mouse = DetectedDevice {
                id: format!("HID\\VID_1D57&PID_{}", &pid[2..].to_ascii_uppercase()),
                name: "HID-compliant mouse".to_string(),
                manufacturer: Some("Microsoft".to_string()),
                vid: Some("0x1d57".to_string()),
                pid: Some(pid.to_string()),
                connection: "USB".to_string(),
                connected: true,
                device_kind: "mouse".to_string(),
            };

            apply_known_mouse_identity(&mut mouse);

            assert_eq!(mouse.name, "DEXP GS Crush");
            assert_eq!(mouse.manufacturer.as_deref(), Some("DEXP"));
            assert_eq!(mouse.connection, connection);
            assert!(is_relevant_mouse(&mouse));
        }
    }
}

#[cfg(target_os = "windows")]
mod windows_device_detection {
    use super::{apply_known_mouse_identity, DetectedDevice};
    use std::mem::size_of;
    use windows::{
        core::PCWSTR,
        Win32::Devices::DeviceAndDriverInstallation::{
            SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW,
            SetupDiGetDeviceInstanceIdW, SetupDiGetDeviceRegistryPropertyW, DIGCF_PRESENT,
            GUID_DEVCLASS_KEYBOARD, GUID_DEVCLASS_MOUSE, HDEVINFO, SETUP_DI_REGISTRY_PROPERTY, SP_DEVINFO_DATA,
            SPDRP_DEVICEDESC, SPDRP_FRIENDLYNAME, SPDRP_HARDWAREID, SPDRP_MFG,
        },
    };

    pub fn detect_mice() -> Result<Vec<DetectedDevice>, String> {
        detect_class(&GUID_DEVCLASS_MOUSE, "mouse", "mouse")
    }

    pub fn detect_keyboards() -> Result<Vec<DetectedDevice>, String> {
        let keyboards = detect_class(&GUID_DEVCLASS_KEYBOARD, "keyboard", "keyboard")?;
        Ok(keyboards.into_iter().filter(|device| {
            device.vid.as_deref() == Some("0x1038") && device.pid.as_deref() == Some("0x1622")
        }).collect())
    }

    fn detect_class(class_guid: &windows::core::GUID, label: &str, device_kind: &str) -> Result<Vec<DetectedDevice>, String> {
        let device_info_set = unsafe { SetupDiGetClassDevsW(Some(class_guid), PCWSTR::null(), None, DIGCF_PRESENT) }
            .map_err(|error| format!("Windows could not enumerate {label} devices: {error}"))?;
        let mut devices = Vec::new();
        let mut index = 0;
        loop {
            let mut device_info = SP_DEVINFO_DATA { cbSize: size_of::<SP_DEVINFO_DATA>() as u32, ..Default::default() };
            if unsafe { SetupDiEnumDeviceInfo(device_info_set, index, &mut device_info) }.is_err() { break; }
            index += 1;
            let instance_id = device_instance_id(device_info_set, &device_info);
            let hardware_id = registry_property(device_info_set, &device_info, SPDRP_HARDWAREID).unwrap_or_else(|| instance_id.clone());
            let name = registry_property(device_info_set, &device_info, SPDRP_FRIENDLYNAME)
                .or_else(|| registry_property(device_info_set, &device_info, SPDRP_DEVICEDESC))
                .unwrap_or_else(|| format!("Unknown {label}"));
            let manufacturer = registry_property(device_info_set, &device_info, SPDRP_MFG).filter(|value| !value.trim().is_empty());
            let id_source = format!("{hardware_id} {instance_id}");
            let mut device = DetectedDevice {
                id: instance_id,
                name,
                manufacturer,
                vid: usb_identifier(&id_source, "VID_"),
                pid: usb_identifier(&id_source, "PID_"),
                connection: connection_type(&id_source).to_string(),
                connected: true,
                device_kind: device_kind.to_string(),
            };
            if device_kind == "mouse" {
                apply_known_mouse_identity(&mut device);
            } else if device.vid.as_deref() == Some("0x1038") && device.pid.as_deref() == Some("0x1622") {
                device.name = "SteelSeries Apex 3 TKL White".to_string();
                device.manufacturer = Some("SteelSeries".to_string());
                device.connection = "Wired USB".to_string();
            }
            devices.push(device);
        }
        let _ = unsafe { SetupDiDestroyDeviceInfoList(device_info_set) };
        devices.sort_by(|left, right| left.name.cmp(&right.name));
        devices.dedup_by(|left, right| left.id == right.id);
        Ok(devices)
    }

    fn registry_property(device_info_set: HDEVINFO, device_info: &SP_DEVINFO_DATA, property: SETUP_DI_REGISTRY_PROPERTY) -> Option<String> {
        let mut required_size = 0;
        let _ = unsafe { SetupDiGetDeviceRegistryPropertyW(device_info_set, device_info, property, None, None, Some(&mut required_size)) };
        if required_size == 0 { return None; }
        let mut buffer = vec![0_u8; required_size as usize];
        unsafe { SetupDiGetDeviceRegistryPropertyW(device_info_set, device_info, property, None, Some(buffer.as_mut_slice()), None).ok()?; }
        let characters = buffer.chunks_exact(2).map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]])).take_while(|character| *character != 0).collect::<Vec<_>>();
        let value = String::from_utf16_lossy(&characters).trim().to_string();
        (!value.is_empty()).then_some(value)
    }

    fn device_instance_id(device_info_set: HDEVINFO, device_info: &SP_DEVINFO_DATA) -> String {
        let mut required_size = 0;
        let _ = unsafe { SetupDiGetDeviceInstanceIdW(device_info_set, device_info, None, Some(&mut required_size)) };
        if required_size == 0 { return "unknown-device".to_string(); }
        let mut buffer = vec![0_u16; required_size as usize];
        if unsafe { SetupDiGetDeviceInstanceIdW(device_info_set, device_info, Some(buffer.as_mut_slice()), None) }.is_err() { return "unknown-device".to_string(); }
        String::from_utf16_lossy(&buffer.into_iter().take_while(|character| *character != 0).collect::<Vec<_>>())
    }

    fn usb_identifier(value: &str, key: &str) -> Option<String> {
        let value = value.to_ascii_uppercase();
        let start = value.find(key)? + key.len();
        let identifier = value[start..].chars().take_while(|character| character.is_ascii_hexdigit()).take(4).collect::<String>();
        (identifier.len() == 4).then(|| format!("0x{}", identifier.to_ascii_lowercase()))
    }

    fn connection_type(value: &str) -> &'static str {
        let value = value.to_ascii_uppercase();
        if value.contains("BTH") || value.contains("BLUETOOTH") { "Bluetooth" }
        else if value.contains("USB") || value.contains("VID_") { "USB" }
        else { "Wired" }
    }
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubReleaseAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadProgress {
    percent: u8,
    status: String,
}

fn emit_download_progress(app: &tauri::AppHandle, percent: u8, status: impl Into<String>) {
    let _ = app.emit("download-progress", DownloadProgress { percent, status: status.into() });
}

fn finish_download_and_launch(
    app: &tauri::AppHandle,
    destination: &PathBuf,
    started_at: Instant,
) -> Result<(), String> {
    const MINIMUM_DOWNLOAD_SCREEN: Duration = Duration::from_secs(8);

    while started_at.elapsed() < MINIMUM_DOWNLOAD_SCREEN {
        let elapsed_ms = started_at.elapsed().as_millis().min(8_000) as u64;
        let percent = 90 + ((elapsed_ms * 9) / 8_000) as u8;
        emit_download_progress(app, percent, "Preparing Unnamed Enhancements...");
        std::thread::sleep(Duration::from_millis(250));
    }

    emit_download_progress(app, 100, "Launching Unnamed Enhancements...");
    Command::new(destination)
        .spawn()
        .map_err(|error| format!("The app downloaded but could not be launched: {error}"))?;
    Ok(())
}

fn bundled_preview_app() -> Option<PathBuf> {
    let current_executable = std::env::current_exe().ok()?;
    let candidate = current_executable.parent()?.join("UnnamedEnhancements.exe");
    candidate.is_file().then_some(candidate)
}

fn launch_bundled_preview(app: &tauri::AppHandle, destination: &PathBuf) -> Result<(), String> {
    const DISPLAY_TIME: Duration = Duration::from_secs(8);
    let started_at = Instant::now();
    while started_at.elapsed() < DISPLAY_TIME {
        let elapsed = started_at.elapsed().as_millis().min(8_000) as u64;
        let percent = ((elapsed * 99) / 8_000) as u8;
        let status = if percent < 18 { "Preparing your app..." } else if percent < 72 { "Setting up Unnamed Enhancements..." } else { "Almost ready..." };
        emit_download_progress(app, percent, status);
        std::thread::sleep(Duration::from_millis(100));
    }
    emit_download_progress(app, 100, "Launching Unnamed Enhancements...");
    Command::new(destination)
        .spawn()
        .map_err(|error| format!("The app is ready but could not be launched: {error}"))?;
    Ok(())
}

#[tauri::command]
fn download_latest_app(app: tauri::AppHandle) -> Result<(), String> {
    // Preview builds ship the app beside the custom downloader. This makes the
    // first-run experience reliable and avoids a Windows installer or a release
    // download that may not exist yet.
    if let Some(destination) = bundled_preview_app() {
        return launch_bundled_preview(&app, &destination);
    }

    let started_at = Instant::now();
    emit_download_progress(&app, 0, "Checking for the latest version...");

    let client = reqwest::blocking::Client::builder()
        .user_agent("UnnamedEnhancementsDownloader/0.1")
        .build()
        .map_err(|error| format!("Could not create download client: {error}"))?;
    let release: GithubRelease = client
        .get("https://api.github.com/repos/7n8s/UnnamedEnhancements/releases/latest")
        .send()
        .map_err(|error| format!("Could not check for a published app release: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Could not check for a published app release: {error}"))?
        .json()
        .map_err(|error| format!("Could not read the published app release: {error}"))?;

    let GithubRelease { tag_name: release_tag, assets } = release;
    let asset = assets
        .into_iter()
        .find(|asset| asset.name == "UnnamedEnhancements.exe")
        .ok_or_else(|| "This downloader needs the Preview bundle, or a published release containing UnnamedEnhancements.exe.".to_string())?;

    let app_data = std::env::var_os("LOCALAPPDATA")
        .ok_or_else(|| "Windows Local AppData could not be found.".to_string())?;
    let install_directory = PathBuf::from(app_data).join("UnnamedEnhancements");
    fs::create_dir_all(&install_directory)
        .map_err(|error| format!("Could not create the app folder: {error}"))?;
    let destination = install_directory.join("UnnamedEnhancements.exe");
    let temporary_destination = install_directory.join("UnnamedEnhancements.download");
    let version_marker = install_directory.join("version.txt");
    let installed_version_matches = fs::read_to_string(&version_marker)
        .map(|version| version.trim() == release_tag)
        .unwrap_or(false);
    let installed_file_matches = fs::metadata(&destination)
        .map(|metadata| metadata.len() == asset.size)
        .unwrap_or(false);

    if installed_version_matches && installed_file_matches {
        emit_download_progress(&app, 90, "Already up to date. Preparing...");
        return finish_download_and_launch(&app, &destination, started_at);
    }

    let mut response = client
        .get(&asset.browser_download_url)
        .send()
        .map_err(|error| format!("Could not start the app download: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Could not start the app download: {error}"))?;
    let total_bytes = response.content_length().filter(|size| *size > 0).unwrap_or(asset.size);
    let mut file = File::create(&temporary_destination)
        .map_err(|error| format!("Could not create the app download: {error}"))?;
    let mut downloaded_bytes = 0_u64;
    let mut last_percent = 0_u8;
    let mut buffer = [0_u8; 256 * 1024];

    loop {
        let read = response.read(&mut buffer).map_err(|error| format!("Could not download the app: {error}"))?;
        if read == 0 { break; }
        file.write_all(&buffer[..read]).map_err(|error| format!("Could not save the app download: {error}"))?;
        downloaded_bytes += read as u64;
        let percent = ((downloaded_bytes.saturating_mul(100) / total_bytes).min(100)) as u8;
        if percent > last_percent {
            last_percent = percent;
            emit_download_progress(&app, percent, format!("Downloading... {percent}%"));
        }
    }
    file.flush().map_err(|error| format!("Could not finish the app download: {error}"))?;
    drop(file);

    if destination.exists() {
        fs::remove_file(&destination).map_err(|error| format!("Close Unnamed Enhancements before updating it: {error}"))?;
    }
    fs::rename(&temporary_destination, &destination).map_err(|error| format!("Could not finish the app download: {error}"))?;
    let _ = fs::write(&version_marker, &release_tag);
    finish_download_and_launch(&app, &destination, started_at)
}

fn main() {
    tauri::Builder::default()
        .manage(TrayState { minimize_to_tray: AtomicBool::new(false) })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![detect_devices, set_apex_rgb, set_apex_rainbow, inspect_dpi_hardware, get_x1_battery, get_dpi, set_dpi, set_minimize_to_tray, get_start_with_windows, set_start_with_windows, test_button_action, apply_button_mappings, download_latest_app])
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } => {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            TrayIconEvent::Click { button: MouseButton::Right, button_state: MouseButtonState::Up, .. } => {
                tray.app_handle().exit(0);
            }
            _ => {}
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.app_handle().state::<TrayState>();
                if state.minimize_to_tray.load(Ordering::Relaxed) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            let mut builder = TrayIconBuilder::new()
                .show_menu_on_left_click(false)
                .tooltip("Unnamed Desktop App");
            if let Some(icon) = app.default_window_icon().cloned() {
                builder = builder.icon(icon);
            }
            builder.build(app)?;
            if std::env::args_os().any(|argument| argument == "--startup") {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
