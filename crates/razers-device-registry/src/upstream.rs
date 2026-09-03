// SPDX-License-Identifier: GPL-2.0-or-later

//! Read-only, source-derived device facts that are not RazeRS support claims.

use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use thiserror::Error;

pub const SUPPORTED_UPSTREAM_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpstreamCatalog {
    pub schema_version: u32,
    pub source: UpstreamSource,
    pub devices: Vec<UpstreamDevice>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpstreamSource {
    pub name: String,
    pub repository: String,
    pub commit: String,
    pub path: String,
    pub license: String,
    pub generated_by: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpstreamDevice {
    pub name: String,
    pub kind: UpstreamDeviceKind,
    pub vid: u16,
    pub pid: u16,
    pub source_path: String,
    pub source_symbol: String,
    #[serde(default)]
    pub upstream_features: Vec<UpstreamFeature>,
    #[serde(default)]
    pub methods: Vec<String>,
    pub matrix: Option<[u16; 2]>,
    pub max_dpi: Option<u32>,
    #[serde(default)]
    pub poll_rates_hz: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum UpstreamDeviceKind {
    Accessory,
    Core,
    Headset,
    Keyboard,
    Keypad,
    Monitor,
    Mouse,
    MouseMat,
}

impl UpstreamDeviceKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accessory => "accessory",
            Self::Core => "core",
            Self::Headset => "headset",
            Self::Keyboard => "keyboard",
            Self::Keypad => "keypad",
            Self::Monitor => "monitor",
            Self::Mouse => "mouse",
            Self::MouseMat => "mouse-mat",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum UpstreamFeature {
    Battery,
    Dpi,
    GameMode,
    Identity,
    Layout,
    Lighting,
    Macro,
    PollingRate,
    ScrollMode,
}

impl UpstreamFeature {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Battery => "battery",
            Self::Dpi => "dpi",
            Self::GameMode => "game-mode",
            Self::Identity => "identity",
            Self::Layout => "layout",
            Self::Lighting => "lighting",
            Self::Macro => "macro",
            Self::PollingRate => "polling-rate",
            Self::ScrollMode => "scroll-mode",
        }
    }
}

pub const SUPPORTED_OPENRGB_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenRgbCatalog {
    pub schema_version: u32,
    pub source: UpstreamSource,
    pub devices: Vec<OpenRgbDevice>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenRgbDevice {
    pub name: String,
    pub kind: OpenRgbDeviceKind,
    pub vid: u16,
    pub pid: u16,
    pub pid_symbol: String,
    pub source_path: String,
    pub source_symbol: String,
    pub matrix_family: MatrixFamily,
    pub transaction_id: u8,
    pub matrix: [u16; 2],
    #[serde(default)]
    pub zones: Vec<String>,
    pub layout: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum OpenRgbDeviceKind {
    Accessory,
    Cooler,
    Gpu,
    Headset,
    HeadsetStand,
    Keyboard,
    Keypad,
    Laptop,
    LedStrip,
    Light,
    Microphone,
    Mouse,
    MouseMat,
    Speaker,
}

impl OpenRgbDeviceKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accessory => "accessory",
            Self::Cooler => "cooler",
            Self::Gpu => "gpu",
            Self::Headset => "headset",
            Self::HeadsetStand => "headset-stand",
            Self::Keyboard => "keyboard",
            Self::Keypad => "keypad",
            Self::Laptop => "laptop",
            Self::LedStrip => "led-strip",
            Self::Light => "light",
            Self::Microphone => "microphone",
            Self::Mouse => "mouse",
            Self::MouseMat => "mouse-mat",
            Self::Speaker => "speaker",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum MatrixFamily {
    Custom,
    Extended,
    ExtendedArgb,
    Linear,
    None,
    Standard,
}

impl MatrixFamily {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Custom => "custom",
            Self::Extended => "extended",
            Self::ExtendedArgb => "extended-argb",
            Self::Linear => "linear",
            Self::None => "none",
            Self::Standard => "standard",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CatalogComparison {
    pub overlap: usize,
    pub openrazer_only: usize,
    pub openrgb_only: usize,
    pub name_differences: usize,
    pub matrix_differences: usize,
}

impl UpstreamCatalog {
    pub fn load_file(path: impl AsRef<Path>) -> Result<Self, UpstreamCatalogError> {
        let path = path.as_ref();
        let source = fs::read_to_string(path).map_err(|source| UpstreamCatalogError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let catalog: Self =
            toml::from_str(&source).map_err(|source| UpstreamCatalogError::Parse {
                path: path.to_path_buf(),
                source,
            })?;
        let problems = catalog.validate();
        if problems.is_empty() {
            Ok(catalog)
        } else {
            Err(UpstreamCatalogError::Validation {
                path: path.to_path_buf(),
                problems,
            })
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if self.schema_version != SUPPORTED_UPSTREAM_SCHEMA_VERSION {
            problems.push(format!(
                "schema_version must be {SUPPORTED_UPSTREAM_SCHEMA_VERSION}, found {}",
                self.schema_version
            ));
        }
        if self.source.name.trim().is_empty()
            || self.source.path.trim().is_empty()
            || self.source.license.trim().is_empty()
            || self.source.generated_by.trim().is_empty()
        {
            problems.push("source name, path, license, and generated_by are required".into());
        }
        if !valid_repository(&self.source.repository) {
            problems.push("source repository must use owner/name form".into());
        }
        if !valid_commit(&self.source.commit) {
            problems.push("source commit must be a full 40-character Git SHA".into());
        }
        if self.devices.is_empty() {
            problems.push("catalog must contain at least one device".into());
        }

        let mut identities = BTreeSet::new();
        for device in &self.devices {
            let identity = (device.vid, device.pid);
            if !identities.insert(identity) {
                problems.push(format!(
                    "duplicate USB identity {:04x}:{:04x}",
                    device.vid, device.pid
                ));
            }
            if device.name.trim().is_empty() || device.source_symbol.trim().is_empty() {
                problems.push(format!(
                    "device {:04x}:{:04x} requires a name and source_symbol",
                    device.vid, device.pid
                ));
            }
            if device.vid == 0 || device.pid == 0 {
                problems.push(format!(
                    "device '{}': VID and PID must be non-zero",
                    device.name
                ));
            }
            if !device.source_path.starts_with(&self.source.path)
                || !device.source_path.ends_with(".py")
            {
                problems.push(format!(
                    "device '{}': source_path must be a Python file below the source root",
                    device.name
                ));
            }
            if device.methods.is_empty() {
                problems.push(format!(
                    "device '{}': at least one upstream method is required",
                    device.name
                ));
            }
            if let Some([rows, columns]) = device.matrix
                && (rows == 0 || columns == 0)
            {
                problems.push(format!(
                    "device '{}': matrix dimensions must be non-zero",
                    device.name
                ));
            }
            if device.max_dpi == Some(0) {
                problems.push(format!(
                    "device '{}': max_dpi must be non-zero",
                    device.name
                ));
            }
            if device.poll_rates_hz.contains(&0) {
                problems.push(format!(
                    "device '{}': poll_rates_hz must not contain zero",
                    device.name
                ));
            }
            let unique_rates = device
                .poll_rates_hz
                .iter()
                .copied()
                .collect::<BTreeSet<_>>();
            if unique_rates.len() != device.poll_rates_hz.len() {
                problems.push(format!(
                    "device '{}': poll_rates_hz contains duplicates",
                    device.name
                ));
            }
        }
        problems
    }

    pub fn find_usb(&self, vid: u16, pid: u16) -> Option<&UpstreamDevice> {
        self.devices
            .iter()
            .find(|device| device.vid == vid && device.pid == pid)
    }

    pub fn compare_openrgb(&self, openrgb: &OpenRgbCatalog) -> CatalogComparison {
        let openrazer_identities = self
            .devices
            .iter()
            .map(|device| ((device.vid, device.pid), device))
            .collect::<std::collections::BTreeMap<_, _>>();
        let openrgb_identities = openrgb
            .devices
            .iter()
            .map(|device| ((device.vid, device.pid), device))
            .collect::<std::collections::BTreeMap<_, _>>();
        let overlap = openrazer_identities
            .keys()
            .filter(|identity| openrgb_identities.contains_key(identity))
            .count();
        let name_differences = openrazer_identities
            .iter()
            .filter_map(|(identity, openrazer)| {
                openrgb_identities
                    .get(identity)
                    .map(|openrgb| (*openrazer, *openrgb))
            })
            .filter(|(openrazer, openrgb)| !openrazer.name.eq_ignore_ascii_case(&openrgb.name))
            .count();
        let matrix_differences = openrazer_identities
            .iter()
            .filter_map(|(identity, openrazer)| {
                openrgb_identities
                    .get(identity)
                    .map(|openrgb| (*openrazer, *openrgb))
            })
            .filter(|(openrazer, openrgb)| {
                openrazer
                    .matrix
                    .is_some_and(|matrix| matrix != openrgb.matrix)
            })
            .count();

        CatalogComparison {
            overlap,
            openrazer_only: openrazer_identities.len() - overlap,
            openrgb_only: openrgb_identities.len() - overlap,
            name_differences,
            matrix_differences,
        }
    }
}

impl OpenRgbCatalog {
    pub fn load_file(path: impl AsRef<Path>) -> Result<Self, OpenRgbCatalogError> {
        let path = path.as_ref();
        let source = fs::read_to_string(path).map_err(|source| OpenRgbCatalogError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let catalog: Self =
            toml::from_str(&source).map_err(|source| OpenRgbCatalogError::Parse {
                path: path.to_path_buf(),
                source,
            })?;
        let problems = catalog.validate();
        if problems.is_empty() {
            Ok(catalog)
        } else {
            Err(OpenRgbCatalogError::Validation {
                path: path.to_path_buf(),
                problems,
            })
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if self.schema_version != SUPPORTED_OPENRGB_SCHEMA_VERSION {
            problems.push(format!(
                "schema_version must be {SUPPORTED_OPENRGB_SCHEMA_VERSION}, found {}",
                self.schema_version
            ));
        }
        validate_source(&self.source, &mut problems);
        if self.devices.is_empty() {
            problems.push("catalog must contain at least one device".into());
        }

        let source_root = Path::new(&self.source.path);
        let mut identities = BTreeSet::new();
        for device in &self.devices {
            if !identities.insert((device.vid, device.pid)) {
                problems.push(format!(
                    "duplicate USB identity {:04x}:{:04x}",
                    device.vid, device.pid
                ));
            }
            if device.vid == 0 || device.pid == 0 {
                problems.push(format!(
                    "device '{}': VID and PID must be non-zero",
                    device.name
                ));
            }
            if device.name.trim().is_empty()
                || device.pid_symbol.trim().is_empty()
                || device.source_symbol.trim().is_empty()
            {
                problems.push(format!(
                    "device {:04x}:{:04x} requires name, pid_symbol, and source_symbol",
                    device.vid, device.pid
                ));
            }
            let source_path = Path::new(&device.source_path);
            if source_path.is_absolute()
                || !source_path.starts_with(source_root)
                || source_path
                    .extension()
                    .is_none_or(|extension| extension != "cpp")
            {
                problems.push(format!(
                    "device '{}': source_path must be a C++ file below the source root",
                    device.name
                ));
            }
            if device.matrix.contains(&0) {
                problems.push(format!(
                    "device '{}': matrix dimensions must be non-zero",
                    device.name
                ));
            }
            let unique_zones = device.zones.iter().collect::<BTreeSet<_>>();
            if unique_zones.len() != device.zones.len() {
                problems.push(format!(
                    "device '{}': zones contains duplicates",
                    device.name
                ));
            }
        }
        problems
    }

    pub fn find_usb(&self, vid: u16, pid: u16) -> Option<&OpenRgbDevice> {
        self.devices
            .iter()
            .find(|device| device.vid == vid && device.pid == pid)
    }
}

fn validate_source(source: &UpstreamSource, problems: &mut Vec<String>) {
    if source.name.trim().is_empty()
        || source.path.trim().is_empty()
        || source.license.trim().is_empty()
        || source.generated_by.trim().is_empty()
    {
        problems.push("source name, path, license, and generated_by are required".into());
    }
    if !valid_repository(&source.repository) {
        problems.push("source repository must use owner/name form".into());
    }
    if !valid_commit(&source.commit) {
        problems.push("source commit must be a full 40-character Git SHA".into());
    }
}

fn valid_commit(commit: &str) -> bool {
    commit.len() == 40 && commit.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_repository(repository: &str) -> bool {
    repository.contains('/') && !repository.contains("://")
}

#[derive(Debug, Error)]
pub enum UpstreamCatalogError {
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
    #[error("invalid upstream catalog '{}': {}", path.display(), problems.join("; "))]
    Validation {
        path: PathBuf,
        problems: Vec<String>,
    },
}

#[derive(Debug, Error)]
pub enum OpenRgbCatalogError {
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
    #[error("invalid OpenRGB catalog '{}': {}", path.display(), problems.join("; "))]
    Validation {
        path: PathBuf,
        problems: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
schema_version = 1

[source]
name = "OpenRazer"
repository = "openrazer/openrazer"
commit = "0123456789abcdef0123456789abcdef01234567"
path = "daemon/openrazer_daemon/hardware"
license = "GPL-2.0-or-later"
generated_by = "tools/import_openrazer.py"

[[devices]]
name = "Razer Test Mouse"
kind = "mouse"
vid = 0x1532
pid = 0x0001
source_path = "daemon/openrazer_daemon/hardware/mouse.py"
source_symbol = "RazerTestMouse"
upstream_features = ["identity", "dpi"]
methods = ["get_device_type_mouse", "get_dpi_xy"]
max_dpi = 16000
"#;

    #[test]
    fn parses_and_matches_a_catalog_entry() {
        let catalog: UpstreamCatalog = toml::from_str(VALID).unwrap();
        assert!(catalog.validate().is_empty());
        assert_eq!(
            catalog.find_usb(0x1532, 0x0001).unwrap().name,
            "Razer Test Mouse"
        );
    }

    #[test]
    fn rejects_duplicate_usb_identities() {
        let source = format!(
            "{VALID}\n[[devices]]{}",
            VALID.split("[[devices]]").nth(1).unwrap()
        );
        let catalog: UpstreamCatalog = toml::from_str(&source).unwrap();
        assert!(
            catalog
                .validate()
                .iter()
                .any(|problem| problem.contains("duplicate USB identity"))
        );
    }

    #[test]
    fn compares_overlapping_catalogs_without_merging_disagreements() {
        let openrazer: UpstreamCatalog = toml::from_str(VALID).unwrap();
        let openrgb: OpenRgbCatalog = toml::from_str(
            r#"
schema_version = 1

[source]
name = "OpenRGB"
repository = "owner/openrgb"
commit = "0123456789abcdef0123456789abcdef01234567"
path = "Controllers/RazerController"
license = "GPL-2.0-or-later"
generated_by = "tools/import_openrgb.py"

[[devices]]
name = "Razer Differently Named Mouse"
kind = "mouse"
vid = 0x1532
pid = 0x0001
pid_symbol = "RAZER_TEST_MOUSE_PID"
source_path = "Controllers/RazerController/RazerDevices.cpp"
source_symbol = "test_mouse_device"
matrix_family = "extended"
transaction_id = 0x1f
matrix = [1, 2]
zones = ["test_mouse_zone"]
"#,
        )
        .unwrap();

        assert!(openrgb.validate().is_empty());
        assert_eq!(
            openrazer.compare_openrgb(&openrgb),
            CatalogComparison {
                overlap: 1,
                openrazer_only: 0,
                openrgb_only: 0,
                name_differences: 1,
                matrix_differences: 0,
            }
        );
    }
}
