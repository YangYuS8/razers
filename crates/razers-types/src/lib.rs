// SPDX-License-Identifier: GPL-2.0-or-later

//! Shared, transport-independent types used across the RazeRS workspace.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Stable identifier for a product-level device descriptor.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct DeviceId(String);

/// Stable identifier for one physical connection belonging to a product.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ConnectionId(String);

/// Why an identifier could not be accepted.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("invalid {kind} identifier '{value}': {reason}")]
pub struct InvalidIdentifier {
    kind: &'static str,
    value: String,
    reason: &'static str,
}

impl DeviceId {
    /// Parse a namespaced product identifier such as `razer.basilisk-v3`.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidIdentifier> {
        validate_identifier("device", value.into(), true).map(Self)
    }

    /// Return the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ConnectionId {
    /// Parse a connection identifier such as `wired` or `receiver-slot-1`.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidIdentifier> {
        validate_identifier("connection", value.into(), false).map(Self)
    }

    /// Return the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn validate_identifier(
    kind: &'static str,
    value: String,
    namespace_required: bool,
) -> Result<String, InvalidIdentifier> {
    let fail = |reason| InvalidIdentifier {
        kind,
        value: value.clone(),
        reason,
    };

    if value.is_empty() {
        return Err(fail("must not be empty"));
    }
    if value.len() > 96 {
        return Err(fail("must be at most 96 bytes"));
    }
    if namespace_required && !value.contains('.') {
        return Err(fail("must contain a namespace separator"));
    }
    if value.starts_with(['.', '-']) || value.ends_with(['.', '-']) {
        return Err(fail("must not start or end with a separator"));
    }
    if value.contains("..") || value.contains("--") || value.contains(".-") || value.contains("-.")
    {
        return Err(fail("must not contain adjacent separators"));
    }
    if !value.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'.' || byte == b'-'
    }) {
        return Err(fail(
            "use lowercase ASCII letters, digits, dots, and hyphens only",
        ));
    }
    if !namespace_required && value.contains('.') {
        return Err(fail("must not contain a namespace separator"));
    }

    Ok(value)
}

macro_rules! identifier_impls {
    ($name:ident) => {
        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = InvalidIdentifier;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }

        impl TryFrom<String> for $name {
            type Error = InvalidIdentifier;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

identifier_impls!(DeviceId);
identifier_impls!(ConnectionId);

/// Confidence level for product or capability support.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SupportStatus {
    Detected,
    Experimental,
    Verified,
    Regressed,
    Unsupported,
}

/// Safety class assigned to every protocol command.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommandRisk {
    ReadOnly,
    Reversible,
    Persistent,
    ExperimentalWrite,
    Firmware,
}

/// Where a setting is expected to persist.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PersistenceScope {
    Session,
    HostProfile,
    DeviceSetting,
    OnboardProfileSlot,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_stable_identifiers() {
        assert_eq!(
            DeviceId::new("razer.basilisk-v3").unwrap().as_str(),
            "razer.basilisk-v3"
        );
        assert_eq!(
            ConnectionId::new("receiver-slot-1").unwrap().as_str(),
            "receiver-slot-1"
        );
    }

    #[test]
    fn rejects_ambiguous_identifiers() {
        assert!(DeviceId::new("BasiliskV3").is_err());
        assert!(DeviceId::new("basilisk-v3").is_err());
        assert!(ConnectionId::new("usb.main").is_err());
        assert!(ConnectionId::new("wired--usb").is_err());
    }
}
