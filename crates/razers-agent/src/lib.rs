// SPDX-License-Identifier: GPL-2.0-or-later

//! Local, privilege-separated device discovery and request handling.
//!
//! The first transport is inherited standard I/O, keeping the Agent private to
//! its parent application while the persistent per-user transport is designed.

use std::{
    collections::BTreeMap,
    io::{self, BufRead, Write},
};

use razers_device_registry::{
    DeviceDescriptor, parse_manifest,
    upstream::{
        EvidenceAgreement, EvidenceAssessment, EvidenceReadiness, IrazerCatalog, OpenRgbCatalog,
        UpstreamCatalog, UpstreamFeature, assess_evidence,
    },
};
use razers_ipc::{
    AgentInfo, DeviceList, DeviceSummary, ERROR_INTERNAL, ERROR_INVALID_PARAMS,
    ERROR_INVALID_REQUEST, ERROR_METHOD_NOT_FOUND, ERROR_PARSE, ERROR_PROTOCOL_VERSION,
    JSON_RPC_VERSION, METHOD_AGENT_INFO, METHOD_DEVICES_LIST, PROTOCOL_VERSION, Request, Response,
    ResponseResult,
};
use razers_transport_hidapi::{HidInterfaceSummary, enumerate_razer};
use razers_types::SupportStatus;
use serde_json::{Value, json};

const OPENRAZER_CATALOG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/upstream/openrazer-devices.toml"
));
const OPENRGB_CATALOG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/upstream/openrgb-devices.toml"
));
const IRAZER_CATALOG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/upstream/irazer-devices.toml"
));
const DEVICE_MANIFESTS: &[(&str, &str)] =
    include!(concat!(env!("OUT_DIR"), "/embedded_devices.rs"));

/// Enumerate connected Razer interfaces without opening hardware.
pub fn discover() -> Result<DeviceList, String> {
    let knowledge = EmbeddedKnowledge::load()?;
    let interfaces = enumerate_razer().map_err(|error| error.to_string())?;
    Ok(knowledge.summarize(&interfaces))
}

/// Handle one already-parsed RazeRS JSON-RPC request.
pub fn handle_request(request: Request) -> Response {
    let id = request.id.clone().unwrap_or(Value::Null);
    if request.jsonrpc != JSON_RPC_VERSION || !valid_id(&id) {
        return Response::failure(Value::Null, ERROR_INVALID_REQUEST, "Invalid Request");
    }
    let Some(protocol_version) = request.protocol_version() else {
        return Response::failure(id, ERROR_INVALID_PARAMS, "Invalid params");
    };
    if protocol_version != PROTOCOL_VERSION {
        return Response::failure_with_data(
            id,
            ERROR_PROTOCOL_VERSION,
            "Unsupported RazeRS protocol version",
            json!({
                "expected": PROTOCOL_VERSION,
                "received": protocol_version,
            }),
        );
    }

    match request.method.as_str() {
        METHOD_AGENT_INFO => Response::success(
            id,
            ResponseResult::AgentInfo(AgentInfo {
                protocol_version: PROTOCOL_VERSION,
                agent_version: env!("CARGO_PKG_VERSION").into(),
                access_mode: "descriptor-only".into(),
                transport: "stdio-child".into(),
            }),
        ),
        METHOD_DEVICES_LIST => match discover() {
            Ok(devices) => Response::success(id, ResponseResult::DeviceList(devices)),
            Err(error) => Response::failure_with_data(
                id,
                ERROR_INTERNAL,
                "Device discovery failed",
                json!({ "detail": error }),
            ),
        },
        _ => Response::failure(id, ERROR_METHOD_NOT_FOUND, "Method not found"),
    }
}

/// Serve newline-delimited JSON-RPC requests until the parent closes stdin.
pub fn serve_stdio(reader: impl BufRead, mut writer: impl Write) -> io::Result<()> {
    for line in reader.lines() {
        if let Some(response) = response_for_line(&line?) {
            serde_json::to_writer(&mut writer, &response)?;
            writer.write_all(b"\n")?;
            writer.flush()?;
        }
    }
    Ok(())
}

fn response_for_line(line: &str) -> Option<Response> {
    let value = match serde_json::from_str::<Value>(line) {
        Ok(value) => value,
        Err(_) => return Some(Response::failure(Value::Null, ERROR_PARSE, "Parse error")),
    };
    let has_id = value
        .as_object()
        .is_some_and(|object| object.contains_key("id"));
    let invalid_request_id = value
        .get("id")
        .filter(|id| valid_id(id))
        .cloned()
        .unwrap_or(Value::Null);
    let request = match serde_json::from_value::<Request>(value) {
        Ok(request) => request,
        Err(_) => {
            return Some(Response::failure(
                invalid_request_id,
                ERROR_INVALID_REQUEST,
                "Invalid Request",
            ));
        }
    };
    let response = handle_request(request);
    has_id.then_some(response)
}

fn valid_id(id: &Value) -> bool {
    id.is_null() || id.is_string() || id.is_number()
}

struct EmbeddedKnowledge {
    manifests: Vec<DeviceDescriptor>,
    openrazer: UpstreamCatalog,
    openrgb: OpenRgbCatalog,
    irazer: IrazerCatalog,
    assessments: BTreeMap<(u16, u16), EvidenceAssessment>,
}

impl EmbeddedKnowledge {
    fn load() -> Result<Self, String> {
        let openrazer: UpstreamCatalog = parse_catalog("OpenRazer", OPENRAZER_CATALOG)?;
        let openrgb: OpenRgbCatalog = parse_catalog("OpenRGB", OPENRGB_CATALOG)?;
        let irazer: IrazerCatalog = parse_catalog("iRazer", IRAZER_CATALOG)?;
        validate_catalog("OpenRazer", openrazer.validate())?;
        validate_catalog("OpenRGB", openrgb.validate())?;
        validate_catalog("iRazer", irazer.validate())?;
        let manifests = DEVICE_MANIFESTS
            .iter()
            .map(|(name, source)| {
                parse_manifest(source, format!("embedded/{name}"))
                    .map_err(|error| format!("unable to load embedded device data: {error}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let assessments = assess_evidence(&openrazer, &openrgb, &irazer)
            .into_iter()
            .map(|assessment| ((assessment.vid, assessment.pid), assessment))
            .collect();

        Ok(Self {
            manifests,
            openrazer,
            openrgb,
            irazer,
            assessments,
        })
    }

    fn summarize(&self, interfaces: &[HidInterfaceSummary]) -> DeviceList {
        let grouped = interfaces.iter().fold(
            BTreeMap::<(u16, u16), Vec<&HidInterfaceSummary>>::new(),
            |mut groups, interface| {
                groups
                    .entry((interface.vendor_id, interface.product_id))
                    .or_default()
                    .push(interface);
                groups
            },
        );
        let devices = grouped
            .into_iter()
            .map(|((vid, pid), interfaces)| self.summarize_device(vid, pid, &interfaces))
            .collect();

        DeviceList {
            protocol_version: PROTOCOL_VERSION,
            devices,
            interface_count: interfaces.len(),
        }
    }

    fn summarize_device(
        &self,
        vid: u16,
        pid: u16,
        interfaces: &[&HidInterfaceSummary],
    ) -> DeviceSummary {
        let manifest = self.manifests.iter().find(|manifest| {
            manifest.connections.iter().any(|connection| {
                interfaces.iter().any(|interface| {
                    connection.identity.matches_hid(
                        interface.vendor_id,
                        interface.product_id,
                        interface.usage_page,
                        interface.usage,
                        interface.interface_number,
                    )
                })
            })
        });
        let assessment = self.assessments.get(&(vid, pid));
        let display_name = manifest
            .map(|manifest| manifest.display_name.clone())
            .or_else(|| local_product_name(interfaces))
            .or_else(|| self.uncontested_upstream_name(vid, pid, assessment))
            .unwrap_or_else(|| "Razer device".into());
        let (support_label, support_detail, capabilities) = manifest.map_or_else(
            || {
                let (support_label, support_detail) = if assessment.is_some() {
                    (
                        "Known device",
                        "RazeRS recognizes this product from community data. Controls are not implemented yet.",
                    )
                } else {
                    (
                        "Unrecognized device",
                        "This Razer product is visible to the operating system but is not in the embedded device catalogs.",
                    )
                };
                (
                    support_label,
                    support_detail,
                    upstream_capabilities(
                        self.openrazer
                            .find_usb(vid, pid)
                            .map(|device| device.upstream_features.as_slice())
                            .unwrap_or_default(),
                    ),
                )
            },
            |manifest| {
                (
                    support_label(manifest.support.status),
                    support_detail(manifest.support.status),
                    manifest.capabilities.names().map(capability_label).collect(),
                )
            },
        );

        DeviceSummary {
            display_name,
            vid,
            pid,
            interface_count: interfaces.len(),
            vendor_interface_count: interfaces
                .iter()
                .filter(|interface| interface.is_vendor_defined_collection())
                .count(),
            support_label: support_label.into(),
            support_detail: support_detail.into(),
            capabilities,
            evidence_label: evidence_label(assessment),
            control_available: false,
        }
    }

    fn uncontested_upstream_name(
        &self,
        vid: u16,
        pid: u16,
        assessment: Option<&EvidenceAssessment>,
    ) -> Option<String> {
        if assessment.is_some_and(|assessment| {
            assessment.sources.len() > 1 && assessment.name_agreement == EvidenceAgreement::Disagree
        }) {
            return None;
        }
        self.openrazer
            .find_usb(vid, pid)
            .map(|device| device.name.clone())
            .or_else(|| {
                self.openrgb
                    .find_usb(vid, pid)
                    .map(|device| device.name.clone())
            })
            .or_else(|| {
                self.irazer
                    .find_usb(vid, pid)
                    .map(|device| device.name.clone())
            })
    }
}

fn parse_catalog<T>(name: &str, source: &str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    toml::from_str(source).map_err(|error| format!("unable to load embedded {name} data: {error}"))
}

fn validate_catalog(name: &str, problems: Vec<String>) -> Result<(), String> {
    if problems.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "embedded {name} data is invalid: {}",
            problems.join("; ")
        ))
    }
}

fn local_product_name(interfaces: &[&HidInterfaceSummary]) -> Option<String> {
    interfaces.iter().find_map(|interface| {
        interface
            .product
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_owned)
    })
}

fn evidence_label(assessment: Option<&EvidenceAssessment>) -> String {
    let Some(assessment) = assessment else {
        return "No imported community record".into();
    };
    match assessment.readiness {
        EvidenceReadiness::Corroborated => format!(
            "Corroborated by {} community sources",
            assessment.sources.len()
        ),
        EvidenceReadiness::NeedsResearch => "Community sources need reconciliation".into(),
        EvidenceReadiness::SingleSource => "Recorded by one community source".into(),
    }
}

fn upstream_capabilities(features: &[UpstreamFeature]) -> Vec<String> {
    features
        .iter()
        .filter_map(|feature| match feature {
            UpstreamFeature::Battery => Some("Battery"),
            UpstreamFeature::Dpi => Some("DPI"),
            UpstreamFeature::GameMode => Some("Game mode"),
            UpstreamFeature::Identity => None,
            UpstreamFeature::Layout => Some("Layout"),
            UpstreamFeature::Lighting => Some("Lighting"),
            UpstreamFeature::Macro => Some("Macros"),
            UpstreamFeature::PollingRate => Some("Polling rate"),
            UpstreamFeature::ScrollMode => Some("Scroll mode"),
        })
        .map(str::to_owned)
        .collect()
}

fn capability_label(capability: &str) -> String {
    match capability {
        "dpi" => "DPI",
        "polling-rate" => "Polling rate",
        "lighting" => "Lighting",
        "battery" => "Battery",
        _ => "Other",
    }
    .into()
}

fn support_label(status: SupportStatus) -> &'static str {
    match status {
        SupportStatus::Detected => "Detected",
        SupportStatus::Experimental => "Experimental",
        SupportStatus::Verified => "Verified",
        SupportStatus::Regressed => "Needs attention",
        SupportStatus::Unsupported => "Unsupported",
    }
}

fn support_detail(status: SupportStatus) -> &'static str {
    match status {
        SupportStatus::Detected => {
            "RazeRS can identify this model. Settings stay locked until its control driver passes the safety checks."
        }
        SupportStatus::Experimental => {
            "Controls are available as an explicit preview and may have documented limitations."
        }
        SupportStatus::Verified => "RazeRS has a recorded hardware test for this device.",
        SupportStatus::Regressed => {
            "This device worked before, but its controls are temporarily unavailable."
        }
        SupportStatus::Unsupported => "RazeRS cannot control this device in its current form.",
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn interface(pid: u16, usage_page: u16, product: Option<&str>) -> HidInterfaceSummary {
        HidInterfaceSummary {
            vendor_id: 0x1532,
            product_id: pid,
            release_number: 0x0100,
            usage_page,
            usage: 1,
            interface_number: 2,
            manufacturer: Some("Razer".into()),
            product: product.map(str::to_owned),
            serial_number_present: true,
        }
    }

    #[test]
    fn embedded_knowledge_is_valid_and_self_contained() {
        let knowledge = EmbeddedKnowledge::load().unwrap();

        assert!(!knowledge.manifests.is_empty());
        assert!(knowledge.assessments.contains_key(&(0x1532, 0x0099)));
    }

    #[test]
    fn groups_interfaces_and_exposes_no_control_before_implementation() {
        let knowledge = EmbeddedKnowledge::load().unwrap();
        let interfaces = [
            interface(0x0099, 0x0001, Some("Basilisk V3")),
            interface(0x0099, 0xff00, Some("Basilisk V3")),
        ];

        let snapshot = knowledge.summarize(&interfaces);

        assert_eq!(snapshot.interface_count, 2);
        assert_eq!(snapshot.devices.len(), 1);
        assert_eq!(snapshot.devices[0].display_name, "Razer Basilisk V3");
        assert_eq!(snapshot.devices[0].interface_count, 2);
        assert_eq!(snapshot.devices[0].vendor_interface_count, 1);
        assert_eq!(snapshot.devices[0].support_label, "Detected");
        assert!(!snapshot.devices[0].control_available);
        assert!(snapshot.devices[0].capabilities.contains(&"DPI".into()));
        assert_eq!(
            snapshot.devices[0].evidence_label,
            "Corroborated by 3 community sources"
        );
    }

    #[test]
    fn keeps_unknown_devices_honest_and_uses_local_product_names() {
        let knowledge = EmbeddedKnowledge::load().unwrap();
        let snapshot = knowledge.summarize(&[interface(0xffff, 0xff00, Some("Prototype Mouse"))]);

        assert_eq!(snapshot.devices[0].display_name, "Prototype Mouse");
        assert_eq!(snapshot.devices[0].support_label, "Unrecognized device");
        assert_eq!(
            snapshot.devices[0].evidence_label,
            "No imported community record"
        );
        assert!(snapshot.devices[0].capabilities.is_empty());
        assert!(!snapshot.devices[0].control_available);
    }

    #[test]
    fn returns_standard_errors_and_suppresses_notification_responses() {
        let input = concat!(
            "not-json\n",
            "[]\n",
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"agent.info\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"missing\",\"params\":{\"protocol_version\":1}}\n",
            "{\"jsonrpc\":\"2.0\",\"method\":\"agent.info\",\"params\":{\"protocol_version\":1}}\n",
        );
        let mut output = Vec::new();

        serve_stdio(Cursor::new(input), &mut output).unwrap();

        let responses = String::from_utf8(output).unwrap();
        let responses = responses.lines().collect::<Vec<_>>();
        let error_codes = responses
            .iter()
            .map(|response| {
                serde_json::from_str::<Response>(response)
                    .unwrap()
                    .error
                    .unwrap()
                    .code
            })
            .collect::<Vec<_>>();
        assert_eq!(
            error_codes,
            [
                ERROR_PARSE,
                ERROR_INVALID_REQUEST,
                ERROR_INVALID_PARAMS,
                ERROR_METHOD_NOT_FOUND,
            ]
        );
    }

    #[test]
    fn reports_versioned_descriptor_only_agent_metadata() {
        let response = handle_request(Request::new(METHOD_AGENT_INFO, json!(1)));
        let Some(ResponseResult::AgentInfo(info)) = response.result else {
            panic!("agent.info must return AgentInfo");
        };

        assert_eq!(info.protocol_version, PROTOCOL_VERSION);
        assert_eq!(info.agent_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(info.access_mode, "descriptor-only");
        assert_eq!(info.transport, "stdio-child");
    }

    #[test]
    fn rejects_incompatible_protocol_versions_before_dispatch() {
        let mut request = Request::new(METHOD_AGENT_INFO, json!(1));
        request.params = Some(json!({ "protocol_version": PROTOCOL_VERSION + 1 }));

        let response = handle_request(request);

        assert_eq!(response.error.unwrap().code, ERROR_PROTOCOL_VERSION);
    }
}
