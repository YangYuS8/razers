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

pub const SUPPORTED_IRAZER_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IrazerCatalog {
    pub schema_version: u32,
    pub source: UpstreamSource,
    pub devices: Vec<IrazerDevice>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IrazerDevice {
    pub source_id: String,
    pub name: String,
    pub kind: OpenRgbDeviceKind,
    pub vid: u16,
    pub pid: u16,
    pub upstream_support: UpstreamSupportClaim,
    #[serde(default)]
    pub capability_labels: Vec<String>,
    pub matrix_family: MatrixFamily,
    pub transaction_id: u8,
    pub source_path: String,
    pub source_symbol: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum UpstreamSupportClaim {
    Experimental,
    Planned,
    Supported,
}

impl UpstreamSupportClaim {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Experimental => "experimental",
            Self::Planned => "planned",
            Self::Supported => "supported",
        }
    }
}

/// One upstream implementation contributing facts about a USB identity.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EvidenceSource {
    OpenRazer,
    OpenRgb,
    Irazer,
}

impl EvidenceSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenRazer => "OpenRazer",
            Self::OpenRgb => "OpenRGB",
            Self::Irazer => "iRazer",
        }
    }
}

/// Whether independent sources can be compared for one fact and agree on it.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EvidenceAgreement {
    NotComparable,
    Agree,
    Disagree,
}

impl EvidenceAgreement {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotComparable => "not-comparable",
            Self::Agree => "agree",
            Self::Disagree => "disagree",
        }
    }
}

/// Review readiness derived from source coverage and recorded disagreements.
///
/// This is a research aid, not an automatic support or live-I/O decision.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EvidenceReadiness {
    Corroborated,
    NeedsResearch,
    SingleSource,
}

impl EvidenceReadiness {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Corroborated => "corroborated",
            Self::NeedsResearch => "needs-research",
            Self::SingleSource => "single-source",
        }
    }
}

/// Field-level assessment for one USB identity across all imported catalogs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceAssessment {
    pub vid: u16,
    pub pid: u16,
    pub sources: Vec<EvidenceSource>,
    pub name_agreement: EvidenceAgreement,
    pub kind_agreement: EvidenceAgreement,
    pub matrix_agreement: EvidenceAgreement,
    pub protocol_agreement: EvidenceAgreement,
    pub readiness: EvidenceReadiness,
    pub openrazer_features: Vec<UpstreamFeature>,
    pub irazer_support: Option<UpstreamSupportClaim>,
}

/// Reconcile source coverage without silently selecting any disputed value.
pub fn assess_evidence(
    openrazer: &UpstreamCatalog,
    openrgb: &OpenRgbCatalog,
    irazer: &IrazerCatalog,
) -> Vec<EvidenceAssessment> {
    let identities = openrazer
        .devices
        .iter()
        .map(|device| (device.vid, device.pid))
        .chain(
            openrgb
                .devices
                .iter()
                .map(|device| (device.vid, device.pid)),
        )
        .chain(irazer.devices.iter().map(|device| (device.vid, device.pid)))
        .collect::<BTreeSet<_>>();

    identities
        .into_iter()
        .map(|(vid, pid)| {
            let openrazer_device = openrazer.find_usb(vid, pid);
            let openrgb_device = openrgb.find_usb(vid, pid);
            let irazer_device = irazer.find_usb(vid, pid);

            let mut sources = Vec::new();
            if openrazer_device.is_some() {
                sources.push(EvidenceSource::OpenRazer);
            }
            if openrgb_device.is_some() {
                sources.push(EvidenceSource::OpenRgb);
            }
            if irazer_device.is_some() {
                sources.push(EvidenceSource::Irazer);
            }

            let names = openrazer_device
                .map(|device| device.name.as_str())
                .into_iter()
                .chain(openrgb_device.map(|device| device.name.as_str()))
                .chain(irazer_device.map(|device| device.name.as_str()))
                .collect::<Vec<_>>();
            let kinds = openrazer_device
                .map(|device| device.kind.as_str())
                .into_iter()
                .chain(openrgb_device.map(|device| device.kind.as_str()))
                .chain(irazer_device.map(|device| device.kind.as_str()))
                .collect::<Vec<_>>();

            let matrix_agreement = match (
                openrazer_device.and_then(|device| device.matrix),
                openrgb_device.map(|device| device.matrix),
            ) {
                (Some(left), Some(right)) if left == right => EvidenceAgreement::Agree,
                (Some(_), Some(_)) => EvidenceAgreement::Disagree,
                _ => EvidenceAgreement::NotComparable,
            };
            let protocol_agreement = match (openrgb_device, irazer_device) {
                (Some(left), Some(right))
                    if left.matrix_family == right.matrix_family
                        && left.transaction_id == right.transaction_id =>
                {
                    EvidenceAgreement::Agree
                }
                (Some(_), Some(_)) => EvidenceAgreement::Disagree,
                _ => EvidenceAgreement::NotComparable,
            };
            let name_agreement = string_agreement(&names, true);
            let kind_agreement = string_agreement(&kinds, false);
            let has_material_disagreement = [kind_agreement, matrix_agreement, protocol_agreement]
                .contains(&EvidenceAgreement::Disagree);
            let readiness = if has_material_disagreement {
                EvidenceReadiness::NeedsResearch
            } else if sources.len() >= 2 {
                EvidenceReadiness::Corroborated
            } else {
                EvidenceReadiness::SingleSource
            };

            EvidenceAssessment {
                vid,
                pid,
                sources,
                name_agreement,
                kind_agreement,
                matrix_agreement,
                protocol_agreement,
                readiness,
                openrazer_features: openrazer_device
                    .map(|device| device.upstream_features.clone())
                    .unwrap_or_default(),
                irazer_support: irazer_device.map(|device| device.upstream_support),
            }
        })
        .collect()
}

fn string_agreement(values: &[&str], case_insensitive: bool) -> EvidenceAgreement {
    let Some((first, rest)) = values.split_first() else {
        return EvidenceAgreement::NotComparable;
    };
    if rest.is_empty() {
        return EvidenceAgreement::NotComparable;
    }
    let agrees = rest.iter().all(|value| {
        if case_insensitive {
            first.eq_ignore_ascii_case(value)
        } else {
            first == value
        }
    });
    if agrees {
        EvidenceAgreement::Agree
    } else {
        EvidenceAgreement::Disagree
    }
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
            if let Some([rows, columns]) = device.matrix {
                if rows == 0 || columns == 0 {
                    problems.push(format!(
                        "device '{}': matrix dimensions must be non-zero",
                        device.name
                    ));
                }
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

impl IrazerCatalog {
    pub fn load_file(path: impl AsRef<Path>) -> Result<Self, IrazerCatalogError> {
        let path = path.as_ref();
        let source = fs::read_to_string(path).map_err(|source| IrazerCatalogError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let catalog: Self =
            toml::from_str(&source).map_err(|source| IrazerCatalogError::Parse {
                path: path.to_path_buf(),
                source,
            })?;
        let problems = catalog.validate();
        if problems.is_empty() {
            Ok(catalog)
        } else {
            Err(IrazerCatalogError::Validation {
                path: path.to_path_buf(),
                problems,
            })
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if self.schema_version != SUPPORTED_IRAZER_SCHEMA_VERSION {
            problems.push(format!(
                "schema_version must be {SUPPORTED_IRAZER_SCHEMA_VERSION}, found {}",
                self.schema_version
            ));
        }
        validate_source(&self.source, &mut problems);
        if self.devices.is_empty() {
            problems.push("catalog must contain at least one device".into());
        }

        let source_root = Path::new(&self.source.path);
        let mut identities = BTreeSet::new();
        let mut source_ids = BTreeSet::new();
        for device in &self.devices {
            if !identities.insert((device.vid, device.pid)) {
                problems.push(format!(
                    "duplicate USB identity {:04x}:{:04x}",
                    device.vid, device.pid
                ));
            }
            if !source_ids.insert(device.source_id.as_str()) {
                problems.push(format!("duplicate source_id '{}'", device.source_id));
            }
            if device.vid == 0 || device.pid == 0 {
                problems.push(format!(
                    "device '{}': VID and PID must be non-zero",
                    device.name
                ));
            }
            if device.name.trim().is_empty()
                || device.source_id.trim().is_empty()
                || device.source_symbol.trim().is_empty()
            {
                problems.push(format!(
                    "device {:04x}:{:04x} requires name, source_id, and source_symbol",
                    device.vid, device.pid
                ));
            }
            let source_path = Path::new(&device.source_path);
            if source_path.is_absolute()
                || !source_path.starts_with(source_root)
                || source_path
                    .extension()
                    .is_none_or(|extension| extension != "swift")
            {
                problems.push(format!(
                    "device '{}': source_path must be a Swift file below the source root",
                    device.name
                ));
            }
            if device.capability_labels.is_empty() {
                problems.push(format!(
                    "device '{}': capability_labels must not be empty",
                    device.name
                ));
            }
            let unique_labels = device
                .capability_labels
                .iter()
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            if unique_labels.len() != device.capability_labels.len() {
                problems.push(format!(
                    "device '{}': capability_labels contains duplicates",
                    device.name
                ));
            }
        }
        problems
    }

    pub fn find_usb(&self, vid: u16, pid: u16) -> Option<&IrazerDevice> {
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

#[derive(Debug, Error)]
pub enum IrazerCatalogError {
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
    #[error("invalid iRazer catalog '{}': {}", path.display(), problems.join("; "))]
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
matrix = [1, 2]
"#;

    const OPENRGB_VALID: &str = r#"
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
"#;

    const IRAZER_VALID: &str = r#"
schema_version = 1

[source]
name = "iRazer"
repository = "owner/irazer"
commit = "0123456789abcdef0123456789abcdef01234567"
path = "Sources/iRazer"
license = "MIT"
generated_by = "tools/import_irazer.py"

[[devices]]
source_id = "test-mouse"
name = "Razer Test Mouse"
kind = "mouse"
vid = 0x1532
pid = 0x0001
upstream_support = "supported"
capability_labels = ["Lighting"]
matrix_family = "extended"
transaction_id = 0x1f
source_path = "Sources/iRazer/DeviceCatalog.swift"
source_symbol = "DeviceCatalog.all:test-mouse"
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
        let openrgb: OpenRgbCatalog = toml::from_str(OPENRGB_VALID).unwrap();

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

    #[test]
    fn corroborates_identity_and_matching_protocol_facts_across_catalogs() {
        let openrazer: UpstreamCatalog = toml::from_str(VALID).unwrap();
        let openrgb: OpenRgbCatalog = toml::from_str(OPENRGB_VALID).unwrap();
        let irazer: IrazerCatalog = toml::from_str(IRAZER_VALID).unwrap();

        let assessments = assess_evidence(&openrazer, &openrgb, &irazer);

        assert_eq!(assessments.len(), 1);
        assert_eq!(
            assessments[0].sources,
            [
                EvidenceSource::OpenRazer,
                EvidenceSource::OpenRgb,
                EvidenceSource::Irazer
            ]
        );
        assert_eq!(assessments[0].name_agreement, EvidenceAgreement::Disagree);
        assert_eq!(assessments[0].kind_agreement, EvidenceAgreement::Agree);
        assert_eq!(assessments[0].matrix_agreement, EvidenceAgreement::Agree);
        assert_eq!(assessments[0].protocol_agreement, EvidenceAgreement::Agree);
        assert_eq!(assessments[0].readiness, EvidenceReadiness::Corroborated);
        assert_eq!(
            assessments[0].irazer_support,
            Some(UpstreamSupportClaim::Supported)
        );
    }

    #[test]
    fn sends_material_disagreements_to_research_without_selecting_a_value() {
        let openrazer: UpstreamCatalog = toml::from_str(VALID).unwrap();
        let openrgb: OpenRgbCatalog =
            toml::from_str(&OPENRGB_VALID.replace("matrix = [1, 2]", "matrix = [2, 1]")).unwrap();
        let irazer: IrazerCatalog = toml::from_str(IRAZER_VALID).unwrap();

        let assessments = assess_evidence(&openrazer, &openrgb, &irazer);

        assert_eq!(assessments[0].matrix_agreement, EvidenceAgreement::Disagree);
        assert_eq!(assessments[0].readiness, EvidenceReadiness::NeedsResearch);
    }

    #[test]
    fn identifies_single_source_records_without_promoting_them() {
        let openrazer: UpstreamCatalog = toml::from_str(VALID).unwrap();
        let openrgb = OpenRgbCatalog {
            schema_version: SUPPORTED_OPENRGB_SCHEMA_VERSION,
            source: openrazer.source.clone(),
            devices: Vec::new(),
        };
        let irazer = IrazerCatalog {
            schema_version: SUPPORTED_IRAZER_SCHEMA_VERSION,
            source: openrazer.source.clone(),
            devices: Vec::new(),
        };

        let assessments = assess_evidence(&openrazer, &openrgb, &irazer);

        assert_eq!(assessments[0].sources, [EvidenceSource::OpenRazer]);
        assert_eq!(assessments[0].readiness, EvidenceReadiness::SingleSource);
        assert_eq!(
            assessments[0].matrix_agreement,
            EvidenceAgreement::NotComparable
        );
    }

    #[test]
    fn preserves_an_upstream_support_label_as_a_source_claim() {
        let catalog: IrazerCatalog = toml::from_str(
            r#"
schema_version = 1

[source]
name = "iRazer"
repository = "owner/irazer"
commit = "0123456789abcdef0123456789abcdef01234567"
path = "Sources/iRazer"
license = "MIT"
generated_by = "tools/import_irazer.py"

[[devices]]
source_id = "nommo-v2"
name = "Razer Nommo V2"
kind = "speaker"
vid = 0x1532
pid = 0x055c
upstream_support = "supported"
capability_labels = ["Brightness", "EQ"]
matrix_family = "extended"
transaction_id = 0x3f
source_path = "Sources/iRazer/DeviceCatalog.swift"
source_symbol = "DeviceCatalog.all:nommo-v2"
"#,
        )
        .unwrap();

        assert!(catalog.validate().is_empty());
        assert_eq!(
            catalog.find_usb(0x1532, 0x055c).unwrap().upstream_support,
            UpstreamSupportClaim::Supported
        );
    }
}
