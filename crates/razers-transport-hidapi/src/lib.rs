// SPDX-License-Identifier: GPL-2.0-or-later

//! Cross-platform HID discovery with privacy-preserving interface summaries.
//!
//! This milestone enumerates descriptors only. It does not open devices or send
//! feature, input, or output reports.

use hidapi::HidApi;
use thiserror::Error;

pub const RAZER_VENDOR_ID: u16 = 0x1532;

/// Non-sensitive information needed to match a HID interface to the registry.
///
/// Device paths and serial-number values are deliberately excluded because they
/// can contain stable identifiers. Callers only learn whether a serial exists.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HidInterfaceSummary {
    pub vendor_id: u16,
    pub product_id: u16,
    pub release_number: u16,
    pub usage_page: u16,
    pub usage: u16,
    pub interface_number: i32,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number_present: bool,
}

/// Enumerate Razer HID interfaces without opening them.
pub fn enumerate_razer() -> Result<Vec<HidInterfaceSummary>, HidEnumerationError> {
    enumerate_vendor(RAZER_VENDOR_ID)
}

/// Enumerate interfaces for one USB vendor without opening them.
pub fn enumerate_vendor(vendor_id: u16) -> Result<Vec<HidInterfaceSummary>, HidEnumerationError> {
    let api = HidApi::new().map_err(HidEnumerationError::Initialize)?;
    let mut interfaces = api
        .device_list()
        .filter(|device| device.vendor_id() == vendor_id)
        .map(|device| HidInterfaceSummary {
            vendor_id: device.vendor_id(),
            product_id: device.product_id(),
            release_number: device.release_number(),
            usage_page: device.usage_page(),
            usage: device.usage(),
            interface_number: device.interface_number(),
            manufacturer: device.manufacturer_string().map(str::to_owned),
            product: device.product_string().map(str::to_owned),
            serial_number_present: device.serial_number().is_some(),
        })
        .collect::<Vec<_>>();
    interfaces.sort();
    Ok(interfaces)
}

#[derive(Debug, Error)]
pub enum HidEnumerationError {
    #[error("unable to initialize the HID subsystem: {0}")]
    Initialize(hidapi::HidError),
}
