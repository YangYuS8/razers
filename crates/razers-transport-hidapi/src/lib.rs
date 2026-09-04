// SPDX-License-Identifier: GPL-2.0-or-later

//! Cross-platform HID discovery with privacy-preserving interface summaries.
//!
//! This milestone enumerates descriptors only. It does not open devices or send
//! feature, input, or output reports.
//!
//! 跨平台、保护隐私的 HID 发现。当前仅枚举描述符，不打开设备，也不发送功能、输入或输出报文。

use hidapi::HidApi;
use thiserror::Error;

pub const RAZER_VENDOR_ID: u16 = 0x1532;

/// Non-sensitive information needed to match a HID interface to the registry.
///
/// Device paths and serial-number values are deliberately excluded because they
/// can contain stable identifiers. Callers only learn whether a serial exists.
///
/// 接口摘要仅保留匹配清单所需的非敏感信息。不包含设备路径或序列号值，只报告序列号是否存在。
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

impl HidInterfaceSummary {
    /// Classify the HID collection using descriptor data only.
    ///
    /// 仅根据描述符数据判断 HID 集合类型。
    pub const fn collection_kind(&self) -> HidCollectionKind {
        HidCollectionKind::from_usage(self.usage_page, self.usage)
    }

    /// Whether this descriptor is a possible vendor-command collection.
    ///
    /// This is only a discovery hint. It does not authorize opening the
    /// interface or sending a report; a curated manifest must still match it.
    ///
    /// 可能的厂商命令集合仅是发现提示，不授权打开接口或发送报文，仍须匹配已审阅清单。
    pub const fn is_vendor_defined_collection(&self) -> bool {
        matches!(self.collection_kind(), HidCollectionKind::VendorDefined)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HidCollectionKind {
    ConsumerControl,
    Keyboard,
    Mouse,
    Other,
    VendorDefined,
}

impl HidCollectionKind {
    pub const fn from_usage(usage_page: u16, usage: u16) -> Self {
        match (usage_page, usage) {
            (0xff00..=0xffff, _) => Self::VendorDefined,
            (0x0001, 0x0002) => Self::Mouse,
            (0x0001, 0x0006) => Self::Keyboard,
            (0x000c, _) => Self::ConsumerControl,
            _ => Self::Other,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ConsumerControl => "consumer-control",
            Self::Keyboard => "keyboard",
            Self::Mouse => "mouse",
            Self::Other => "other",
            Self::VendorDefined => "vendor-defined",
        }
    }
}

/// Enumerate Razer HID interfaces without opening them.
///
/// 枚举 Razer HID 接口，不打开设备。
pub fn enumerate_razer() -> Result<Vec<HidInterfaceSummary>, HidEnumerationError> {
    enumerate_vendor(RAZER_VENDOR_ID)
}

/// Enumerate interfaces for one USB vendor without opening them.
///
/// 枚举指定 USB 厂商的 HID 接口，不打开设备。
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinguishes_input_and_vendor_defined_collections() {
        assert_eq!(
            HidCollectionKind::from_usage(0x0001, 0x0002),
            HidCollectionKind::Mouse
        );
        assert_eq!(
            HidCollectionKind::from_usage(0x0001, 0x0006),
            HidCollectionKind::Keyboard
        );
        assert_eq!(
            HidCollectionKind::from_usage(0xff01, 0x0001),
            HidCollectionKind::VendorDefined
        );
    }

    #[test]
    fn never_treats_standard_input_as_a_vendor_collection() {
        for (usage_page, usage) in [(0x0001, 0x0002), (0x0001, 0x0006), (0x000c, 0x0001)] {
            assert_ne!(
                HidCollectionKind::from_usage(usage_page, usage),
                HidCollectionKind::VendorDefined
            );
        }
    }
}
