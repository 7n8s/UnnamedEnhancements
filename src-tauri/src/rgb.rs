use hidapi::{HidApi, HidDevice};
use serde::Deserialize;

const STEELSERIES_VENDOR_ID: u16 = 0x1038;
const APEX_3_TKL_PRODUCT_ID: u16 = 0x1622;
const RGB_INTERFACE: i32 = 1;
const RGB_USAGE_PAGE: u16 = 0xffc0;
const RGB_USAGE: u16 = 0x0001;
const ZONE_COUNT: usize = 8;

#[derive(Debug, Clone, Deserialize)]
pub struct RgbColor {
    red: u8,
    green: u8,
    blue: u8,
}

fn apex_handle(api: &HidApi) -> Result<HidDevice, String> {
    let path = api
        .device_list()
        .find(|device| {
            device.vendor_id() == STEELSERIES_VENDOR_ID
                && device.product_id() == APEX_3_TKL_PRODUCT_ID
                && device.interface_number() == RGB_INTERFACE
                && device.usage_page() == RGB_USAGE_PAGE
                && device.usage() == RGB_USAGE
        })
        .or_else(|| {
            api.device_list().find(|device| {
                device.vendor_id() == STEELSERIES_VENDOR_ID
                    && device.product_id() == APEX_3_TKL_PRODUCT_ID
                    && device.interface_number() == RGB_INTERFACE
            })
        })
        .map(|device| device.path().to_owned())
        .ok_or_else(|| "The Apex 3 TKL lighting interface was not found. Reconnect the keyboard and close SteelSeries GG before trying again.".to_string())?;

    api.open_path(&path)
        .map_err(|error| format!("Could not open the Apex 3 TKL lighting interface: {error}. Close SteelSeries GG and try again."))
}

fn write_report(device: &HidDevice, command: u8, payload: &[u8]) -> Result<(), String> {
    let mut report = [0_u8; 65];
    report[1] = command;
    let length = payload.len().min(63);
    report[2..2 + length].copy_from_slice(&payload[..length]);
    device
        .write(&report)
        .map_err(|error| format!("Could not send the Apex lighting command: {error}"))?;
    Ok(())
}

fn set_brightness(device: &HidDevice, brightness: u8) -> Result<(), String> {
    write_report(device, 0x23, &[brightness.min(16)])
}

pub fn set_apex_rgb(colors: &[RgbColor], brightness: u8) -> Result<(), String> {
    if colors.len() != ZONE_COUNT {
        return Err(format!("Apex 3 TKL lighting requires exactly {ZONE_COUNT} zone colours."));
    }
    let api = HidApi::new().map_err(|error| format!("Could not initialise USB lighting control: {error}"))?;
    let device = apex_handle(&api)?;
    set_brightness(&device, brightness)?;

    let mut payload = Vec::with_capacity(1 + ZONE_COUNT * 3);
    payload.push(0xff);
    for color in colors {
        payload.extend_from_slice(&[color.red, color.green, color.blue]);
    }
    write_report(&device, 0x21, &payload)
}

pub fn set_apex_rainbow(brightness: u8) -> Result<(), String> {
    let api = HidApi::new().map_err(|error| format!("Could not initialise USB lighting control: {error}"))?;
    let device = apex_handle(&api)?;
    set_brightness(&device, brightness)?;
    write_report(&device, 0x22, &[0xff])
}
