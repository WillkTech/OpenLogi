//! Discovery of generic non-HID++ USB and Bluetooth mice.

use openlogi_core::device::{GenericMouseDevice, RawDeviceAddress};

use super::InventoryError;
use crate::backend::HidBackend;
use crate::write::matches_litra;

/// Whether `(usage_page, usage_id, vendor_id, product_id)` matches a standard HID generic mouse.
#[must_use]
pub fn is_generic_mouse_candidate(
    usage_page: u16,
    usage_id: u16,
    vendor_id: u16,
    product_id: u16,
) -> bool {
    let is_mouse_usage = usage_page == 0x0001 && (usage_id == 0x0002 || usage_id == 0x0001);
    let is_hidpp_page = usage_page == 0xff00 || usage_page == 0xff43;

    is_mouse_usage
        && !is_hidpp_page
        && !matches_litra(vendor_id, product_id, usage_page, usage_id)
}

/// Whether a device is a Logitech G90 gaming mouse (`VID: 0x046D`, `PID: 0xC07E` / `0xC24F` / `0xC07F`).
#[must_use]
pub fn is_logitech_g90(vendor_id: u16, product_id: u16) -> bool {
    vendor_id == 0x046d
        && (product_id == 0xc07e || product_id == 0xc07f || product_id == 0xc24f)
}

/// Enumerate generic USB/Bluetooth mice directly without initiating HID++ probing.
pub async fn enumerate_generic_mice(
    backend: &dyn HidBackend,
) -> Result<Vec<GenericMouseDevice>, InventoryError> {
    let devices = backend.enumerate().await?;
    let generic_mice = devices
        .into_iter()
        .filter_map(|device| {
            if !is_generic_mouse_candidate(
                device.usage_page,
                device.usage_id,
                device.vendor_id,
                device.product_id,
            ) {
                return None;
            }

            let display_name = if is_logitech_g90(device.vendor_id, device.product_id) {
                if !device.name.trim().is_empty() {
                    device.name.clone()
                } else {
                    "Logitech G90 Gaming Mouse".to_string()
                }
            } else if !device.name.trim().is_empty() {
                device.name.clone()
            } else {
                "Generic Mouse".to_string()
            };

            let identity = device.identity();
            Some(GenericMouseDevice {
                address: RawDeviceAddress {
                    vendor_id: device.vendor_id,
                    product_id: device.product_id,
                    usage_page: device.usage_page,
                    usage_id: device.usage_id,
                    identity,
                },
                vendor_id: device.vendor_id,
                product_id: device.product_id,
                display_name,
                manufacturer: device.manufacturer.clone(),
                serial_number: device.serial_number.clone(),
                online: true,
            })
        })
        .collect();

    Ok(generic_mice)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_mouse_candidate_matches_standard_desktop_mouse() {
        assert!(is_generic_mouse_candidate(0x0001, 0x0002, 0x1234, 0x5678));
        assert!(is_generic_mouse_candidate(0x0001, 0x0001, 0x1234, 0x5678));
    }

    #[test]
    fn generic_mouse_candidate_excludes_hidpp_and_litra() {
        // HID++ usage page
        assert!(!is_generic_mouse_candidate(0xff00, 0x0002, 0x046d, 0xc52b));
        assert!(!is_generic_mouse_candidate(0xff43, 0x0202, 0x046d, 0xc548));
        // Litra Glow
        assert!(!is_generic_mouse_candidate(0xff43, 0x0202, 0x046d, 0xc900));
    }

    #[test]
    fn logitech_g90_is_recognized() {
        assert!(is_logitech_g90(0x046d, 0xc07e));
        assert!(is_logitech_g90(0x046d, 0xc24f));
        assert!(!is_logitech_g90(0x046d, 0xc52b));
    }
}
