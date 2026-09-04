// SPDX-License-Identifier: GPL-2.0-or-later

//! Versioned JSON-RPC 2.0 messages shared by the RazeRS Agent and clients.
//!
//! RazeRS Agent 与客户端共用的版本化 JSON-RPC 2.0 消息。
//!
//! # Example / 示例
//!
//! Construct a versioned request without launching an Agent or accessing hardware.
//! 无需启动 Agent 或访问硬件，即可构建带版本信息的请求。
//!
//! ```
//! use razers_ipc::{Request, METHOD_AGENT_INFO, PROTOCOL_VERSION};
//! use serde_json::json;
//!
//! let request = Request::new(METHOD_AGENT_INFO, json!(1));
//! assert_eq!(request.protocol_version(), Some(PROTOCOL_VERSION));
//! let wire = serde_json::to_string(&request)?;
//! let decoded: Request = serde_json::from_str(&wire)?;
//! assert_eq!(decoded.method, METHOD_AGENT_INFO);
//! # Ok::<(), serde_json::Error>(())
//! ```

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const JSON_RPC_VERSION: &str = "2.0";
pub const PROTOCOL_VERSION: u32 = 1;
pub const METHOD_AGENT_INFO: &str = "agent.info";
pub const METHOD_DEVICES_LIST: &str = "devices.list";

pub const ERROR_PARSE: i32 = -32700;
pub const ERROR_INVALID_REQUEST: i32 = -32600;
pub const ERROR_METHOD_NOT_FOUND: i32 = -32601;
pub const ERROR_INVALID_PARAMS: i32 = -32602;
pub const ERROR_INTERNAL: i32 = -32603;
pub const ERROR_PROTOCOL_VERSION: i32 = -32001;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Request {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

impl Request {
    pub fn new(method: impl Into<String>, id: Value) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.into(),
            method: method.into(),
            params: Some(json!({ "protocol_version": PROTOCOL_VERSION })),
            id: Some(id),
        }
    }

    pub fn protocol_version(&self) -> Option<u32> {
        self.params
            .as_ref()?
            .as_object()?
            .get("protocol_version")?
            .as_u64()?
            .try_into()
            .ok()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Response {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<ResponseResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

impl Response {
    pub fn success(id: Value, result: ResponseResult) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn failure(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.into(),
            id,
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }

    pub fn failure_with_data(
        id: Value,
        code: i32,
        message: impl Into<String>,
        data: Value,
    ) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.into(),
            id,
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
                data: Some(data),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ResponseResult {
    AgentInfo(AgentInfo),
    DeviceList(DeviceList),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentInfo {
    pub protocol_version: u32,
    pub agent_version: String,
    pub access_mode: String,
    pub transport: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DeviceList {
    pub protocol_version: u32,
    pub devices: Vec<DeviceSummary>,
    pub interface_count: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DeviceSummary {
    pub display_name: String,
    pub vid: u16,
    pub pid: u16,
    pub interface_count: usize,
    pub vendor_interface_count: usize,
    pub support_label: String,
    pub support_detail: String,
    pub capabilities: Vec<String>,
    pub evidence_label: String,
    /// Additive v1 metadata for localized evidence counts; old Agents omit it.
    ///
    /// v1 的可选扩展字段，用于本地化来源数量；旧版 Agent 不提供该字段。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_source_count: Option<usize>,
    pub control_available: bool,
}

impl DeviceSummary {
    pub fn usb_identity(&self) -> String {
        format!("{:04X}:{:04X}", self.vid, self.pid)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_a_versioned_json_rpc_request() {
        let request = Request::new(METHOD_DEVICES_LIST, json!(7));
        let encoded = serde_json::to_value(&request).unwrap();

        assert_eq!(encoded["jsonrpc"], JSON_RPC_VERSION);
        assert_eq!(encoded["id"], 7);
        assert_eq!(encoded["params"]["protocol_version"], PROTOCOL_VERSION);
        assert_eq!(request.protocol_version(), Some(PROTOCOL_VERSION));
    }

    #[test]
    fn accepts_legacy_device_summaries_without_localization_metadata() {
        let old = json!({
            "display_name": "Razer Test", "vid": 5426, "pid": 153,
            "interface_count": 1, "vendor_interface_count": 1,
            "support_label": "Detected", "support_detail": "Legacy detail",
            "capabilities": ["DPI"], "evidence_label": "Legacy evidence",
            "control_available": false
        });
        let mut device: DeviceSummary = serde_json::from_value(old.clone()).unwrap();
        assert_eq!(device.evidence_source_count, None);
        assert_eq!(serde_json::to_value(&device).unwrap(), old);
        device.evidence_source_count = Some(3);
        assert_eq!(
            serde_json::to_value(&device).unwrap()["evidence_source_count"],
            3
        );
    }

    #[test]
    fn success_and_error_responses_are_mutually_exclusive() {
        let success = Response::success(
            json!(1),
            ResponseResult::AgentInfo(AgentInfo {
                protocol_version: PROTOCOL_VERSION,
                agent_version: "0.1.0".into(),
                access_mode: "descriptor-only".into(),
                transport: "stdio-child".into(),
            }),
        );
        let failure = Response::failure(Value::Null, ERROR_PARSE, "Parse error");

        assert!(success.result.is_some());
        assert!(success.error.is_none());
        assert!(failure.result.is_none());
        assert!(failure.error.is_some());
    }
}
