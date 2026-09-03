// SPDX-License-Identifier: GPL-2.0-or-later

//! Versioned, declarative device manifests and their validation rules.

pub mod upstream;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

use razers_types::{ConnectionId, DeviceId, PersistenceScope, SupportStatus};
use serde::Deserialize;
use thiserror::Error;

pub const SUPPORTED_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceDescriptor {
    pub schema_version: u32,
    pub id: DeviceId,
    pub display_name: String,
    pub kind: DeviceKind,
    pub support: SupportDescriptor,
    pub connections: Vec<ConnectionDescriptor>,
    pub capabilities: Capabilities,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    #[serde(default)]
    pub verification: Vec<Verification>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum DeviceKind {
    Mouse,
    Keyboard,
    Headset,
    Speaker,
    MouseMat,
    Laptop,
    Receiver,
    Accessory,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupportDescriptor {
    pub status: SupportStatus,
    pub notes: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionDescriptor {
    pub id: ConnectionId,
    pub role: String,
    pub transport: TransportKind,
    #[serde(rename = "match")]
    pub identity: MatchDescriptor,
    pub protocol: ProtocolDescriptor,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TransportKind {
    UsbHidFeature,
    UsbHidOutput,
    BleGatt,
    OsAudio,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchDescriptor {
    pub vid: u16,
    pub pid: u16,
    pub usage_page: Option<u16>,
    pub usage: Option<u16>,
    pub interface_number: Option<u8>,
}

impl MatchDescriptor {
    /// Return whether a privacy-preserving HID descriptor matches this connection.
    pub fn matches_hid(
        &self,
        vendor_id: u16,
        product_id: u16,
        usage_page: u16,
        usage: u16,
        interface_number: i32,
    ) -> bool {
        self.vid == vendor_id
            && self.pid == product_id
            && self
                .usage_page
                .is_none_or(|expected| expected == usage_page)
            && self.usage.is_none_or(|expected| expected == usage)
            && self
                .interface_number
                .is_none_or(|expected| i32::from(expected) == interface_number)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolDescriptor {
    pub family: String,
    pub report_id: u8,
    pub transaction_id: u8,
    pub response_delay_us: u64,
    pub busy_retries: u8,
    #[serde(default)]
    pub quirks: ProtocolQuirks,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProtocolQuirks {
    pub include_report_id_in_payload: bool,
    pub validate_response_crc: bool,
    pub validate_command_echo: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Capabilities {
    pub dpi: Option<DpiCapability>,
    pub polling_rate: Option<PollingRateCapability>,
    pub lighting: Option<LightingCapability>,
    pub battery: Option<BatteryCapability>,
}

impl Capabilities {
    pub fn count(&self) -> usize {
        [
            self.dpi.is_some(),
            self.polling_rate.is_some(),
            self.lighting.is_some(),
            self.battery.is_some(),
        ]
        .into_iter()
        .filter(|present| *present)
        .count()
    }

    pub fn names(&self) -> impl Iterator<Item = &'static str> {
        [
            self.dpi.as_ref().map(|_| "dpi"),
            self.polling_rate.as_ref().map(|_| "polling-rate"),
            self.lighting.as_ref().map(|_| "lighting"),
            self.battery.as_ref().map(|_| "battery"),
        ]
        .into_iter()
        .flatten()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DpiCapability {
    pub status: SupportStatus,
    pub driver: String,
    pub minimum: u32,
    pub maximum: u32,
    pub step: u32,
    pub axes: DpiAxes,
    #[serde(default)]
    pub persistence: Vec<PersistenceScope>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum DpiAxes {
    Single,
    Xy,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PollingRateCapability {
    pub status: SupportStatus,
    pub driver: String,
    #[serde(default)]
    pub values_hz: Vec<u32>,
    #[serde(default)]
    pub persistence: Vec<PersistenceScope>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LightingCapability {
    pub status: SupportStatus,
    pub driver: String,
    pub rows: u16,
    pub columns: u16,
    pub effects: Vec<String>,
    #[serde(default)]
    pub zones: Vec<LightingZone>,
    #[serde(default)]
    pub persistence: Vec<PersistenceScope>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LightingZone {
    pub id: String,
    pub protocol_led_id: u8,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatteryCapability {
    pub status: SupportStatus,
    pub driver: String,
    pub read_only: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub source: String,
    pub repository: String,
    pub commit: String,
    pub path: String,
    pub symbol: String,
    pub license: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Verification {
    pub platform: String,
    pub firmware: String,
    pub result: VerificationResult,
    #[serde(default)]
    pub capabilities: Vec<String>,
    pub notes: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum VerificationResult {
    Passed,
    Partial,
    Failed,
}

#[derive(Clone, Debug)]
pub struct Registry {
    devices: BTreeMap<DeviceId, LoadedDevice>,
}

#[derive(Clone, Debug)]
pub struct LoadedDevice {
    pub source_path: PathBuf,
    pub descriptor: DeviceDescriptor,
}

impl Registry {
    pub fn load_dir(path: impl AsRef<Path>) -> Result<Self, RegistryError> {
        let path = path.as_ref();
        let mut manifest_paths = fs::read_dir(path)
            .map_err(|source| RegistryError::Io {
                path: path.to_path_buf(),
                source,
            })?
            .map(|entry| {
                entry
                    .map(|entry| entry.path())
                    .map_err(|source| RegistryError::Io {
                        path: path.to_path_buf(),
                        source,
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        manifest_paths.retain(|path| {
            path.extension()
                .is_some_and(|extension| extension == "toml")
        });
        manifest_paths.sort();

        let mut devices = BTreeMap::new();
        for source_path in manifest_paths {
            let source = fs::read_to_string(&source_path).map_err(|source| RegistryError::Io {
                path: source_path.clone(),
                source,
            })?;
            let descriptor = parse_manifest(&source, &source_path)?;
            let id = descriptor.id.clone();
            if let Some(previous) = devices.insert(
                id.clone(),
                LoadedDevice {
                    source_path: source_path.clone(),
                    descriptor,
                },
            ) {
                return Err(RegistryError::DuplicateDevice {
                    id,
                    first: previous.source_path,
                    second: source_path,
                });
            }
        }

        Ok(Self { devices })
    }

    pub fn len(&self) -> usize {
        self.devices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &LoadedDevice> {
        self.devices.values()
    }

    pub fn get(&self, id: &DeviceId) -> Option<&LoadedDevice> {
        self.devices.get(id)
    }
}

pub fn parse_manifest(
    source: &str,
    source_path: impl AsRef<Path>,
) -> Result<DeviceDescriptor, RegistryError> {
    let source_path = source_path.as_ref();
    let descriptor: DeviceDescriptor =
        toml::from_str(source).map_err(|source| RegistryError::Parse {
            path: source_path.to_path_buf(),
            source,
        })?;
    let problems = validate(&descriptor);
    if problems.is_empty() {
        Ok(descriptor)
    } else {
        Err(RegistryError::Validation {
            path: source_path.to_path_buf(),
            problems,
        })
    }
}

pub fn validate(descriptor: &DeviceDescriptor) -> Vec<String> {
    let mut problems = Vec::new();

    if descriptor.schema_version != SUPPORTED_SCHEMA_VERSION {
        problems.push(format!(
            "schema_version must be {SUPPORTED_SCHEMA_VERSION}, found {}",
            descriptor.schema_version
        ));
    }
    if descriptor.display_name.trim().is_empty() {
        problems.push("display_name must not be empty".into());
    }
    if descriptor.support.notes.trim().is_empty() {
        problems.push("support.notes must explain the current support state".into());
    }
    if descriptor.connections.is_empty() {
        problems.push("at least one connection is required".into());
    }
    if descriptor.capabilities.count() == 0 {
        problems.push("at least one capability is required".into());
    }
    if descriptor.evidence.is_empty() {
        problems.push("at least one pinned evidence source is required".into());
    }
    if descriptor.support.status == SupportStatus::Verified && descriptor.verification.is_empty() {
        problems.push("verified devices require at least one verification record".into());
    }

    let mut connection_ids = BTreeSet::new();
    for connection in &descriptor.connections {
        if !connection_ids.insert(connection.id.clone()) {
            problems.push(format!("duplicate connection id '{}'", connection.id));
        }
        if connection.role.trim().is_empty() {
            problems.push(format!(
                "connection '{}': role must not be empty",
                connection.id
            ));
        }
        if connection.identity.vid == 0 || connection.identity.pid == 0 {
            problems.push(format!(
                "connection '{}': VID and PID must be non-zero",
                connection.id
            ));
        }
        if connection.protocol.family.trim().is_empty() {
            problems.push(format!(
                "connection '{}': protocol family must not be empty",
                connection.id
            ));
        }
        if connection.protocol.response_delay_us > 10_000_000 {
            problems.push(format!(
                "connection '{}': response_delay_us exceeds the 10 second safety limit",
                connection.id
            ));
        }
        if connection.protocol.busy_retries > 20 {
            problems.push(format!(
                "connection '{}': busy_retries exceeds the limit of 20",
                connection.id
            ));
        }
    }

    if let Some(dpi) = &descriptor.capabilities.dpi {
        if dpi.minimum == 0 || dpi.minimum > dpi.maximum {
            problems.push("capabilities.dpi must have 0 < minimum <= maximum".into());
        }
        if dpi.step == 0 {
            problems.push("capabilities.dpi.step must be non-zero".into());
        }
        if dpi.driver.trim().is_empty() {
            problems.push("capabilities.dpi.driver must not be empty".into());
        }
    }

    if let Some(polling_rate) = &descriptor.capabilities.polling_rate {
        if polling_rate.driver.trim().is_empty() {
            problems.push("capabilities.polling_rate.driver must not be empty".into());
        }
        if polling_rate.values_hz.contains(&0) {
            problems.push("capabilities.polling_rate.values_hz must not contain zero".into());
        }
        let unique = polling_rate
            .values_hz
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        if unique.len() != polling_rate.values_hz.len() {
            problems.push("capabilities.polling_rate.values_hz contains duplicates".into());
        }
    }

    if let Some(lighting) = &descriptor.capabilities.lighting {
        if lighting.driver.trim().is_empty() {
            problems.push("capabilities.lighting.driver must not be empty".into());
        }
        if lighting.rows == 0 || lighting.columns == 0 {
            problems.push("capabilities.lighting dimensions must be non-zero".into());
        }
        if lighting.effects.is_empty() {
            problems.push("capabilities.lighting.effects must not be empty".into());
        }
        let mut zone_ids = BTreeSet::new();
        for zone in &lighting.zones {
            if zone.id.trim().is_empty() {
                problems.push("lighting zone id must not be empty".into());
            } else if !zone_ids.insert(&zone.id) {
                problems.push(format!("duplicate lighting zone id '{}'", zone.id));
            }
        }
    }

    if let Some(battery) = &descriptor.capabilities.battery {
        if battery.driver.trim().is_empty() {
            problems.push("capabilities.battery.driver must not be empty".into());
        }
    }

    for evidence in &descriptor.evidence {
        if evidence.commit.len() != 40
            || !evidence.commit.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            problems.push(format!(
                "evidence source '{}': commit must be a full 40-character Git SHA",
                evidence.source
            ));
        }
        if !evidence.repository.contains('/') || evidence.repository.contains("://") {
            problems.push(format!(
                "evidence source '{}': repository must use owner/name form",
                evidence.source
            ));
        }
        if evidence.path.trim().is_empty()
            || evidence.symbol.trim().is_empty()
            || evidence.license.trim().is_empty()
        {
            problems.push(format!(
                "evidence source '{}': path, symbol, and license are required",
                evidence.source
            ));
        }
    }

    problems
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("unable to read '{}': {source}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("unable to parse '{}': {source}", path.display())]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("invalid device manifest '{}': {}", path.display(), problems.join("; "))]
    Validation {
        path: PathBuf,
        problems: Vec<String>,
    },
    #[error("device '{id}' is declared by both '{}' and '{}'", first.display(), second.display())]
    DuplicateDevice {
        id: DeviceId,
        first: PathBuf,
        second: PathBuf,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
schema_version = 1
id = "razer.test-device"
display_name = "Razer Test Device"
kind = "mouse"

[support]
status = "detected"
notes = "Schema test only."

[[connections]]
id = "wired"
role = "control"
transport = "usb-hid-feature"

[connections.match]
vid = 5426
pid = 1

[connections.protocol]
family = "razer-report-90"
report_id = 0
transaction_id = 0
response_delay_us = 600
busy_retries = 5

[capabilities.dpi]
status = "experimental"
driver = "dpi-u16-xy"
minimum = 100
maximum = 1000
step = 50
axes = "xy"

[[evidence]]
source = "test"
repository = "owner/repository"
commit = "0123456789abcdef0123456789abcdef01234567"
path = "tests/device.toml"
symbol = "test"
license = "GPL-2.0-or-later"
"#;

    #[test]
    fn parses_and_validates_a_manifest() {
        let descriptor = parse_manifest(VALID, "test.toml").unwrap();
        assert_eq!(descriptor.id.as_str(), "razer.test-device");
        assert_eq!(descriptor.capabilities.count(), 1);
    }

    #[test]
    fn reports_multiple_semantic_problems() {
        let invalid = VALID
            .replace("schema_version = 1", "schema_version = 2")
            .replace("minimum = 100", "minimum = 2000");

        let RegistryError::Validation { problems, .. } =
            parse_manifest(&invalid, "invalid.toml").unwrap_err()
        else {
            panic!("expected validation failure")
        };
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("schema_version"))
        );
        assert!(problems.iter().any(|problem| problem.contains("minimum")));
    }

    #[test]
    fn matches_only_the_declared_hid_interface_constraints() {
        let identity = MatchDescriptor {
            vid: 0x1532,
            pid: 0x0099,
            usage_page: Some(0xff00),
            usage: None,
            interface_number: Some(2),
        };

        assert!(identity.matches_hid(0x1532, 0x0099, 0xff00, 1, 2));
        assert!(!identity.matches_hid(0x1532, 0x0099, 0x0001, 1, 2));
        assert!(!identity.matches_hid(0x1532, 0x0099, 0xff00, 1, 1));
        assert!(!identity.matches_hid(0x1234, 0x0099, 0xff00, 1, 2));
    }
}
