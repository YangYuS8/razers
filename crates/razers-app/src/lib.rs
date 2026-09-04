// SPDX-License-Identifier: GPL-2.0-or-later

//! Private child-process IPC client used by the RazeRS desktop application.
//!
//! RazeRS 桌面应用使用的私有子进程 IPC 客户端。

use std::{
    env,
    ffi::OsString,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use razers_ipc::{
    DeviceList, JSON_RPC_VERSION, METHOD_DEVICES_LIST, PROTOCOL_VERSION, Request, Response,
    ResponseResult,
};
use serde_json::json;

/// A localized summary plus untranslated technical detail; never a hardware command.
/// 可翻译的错误摘要与保留原样的技术详情。
#[derive(Clone, Debug)]
pub struct DiscoveryError {
    pub message: &'static str,
    pub detail: String,
}
impl DiscoveryError {
    fn new(message: &'static str, detail: impl std::fmt::Display) -> Self {
        Self {
            message,
            detail: detail.to_string(),
        }
    }
}
impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.message, self.detail)
    }
}
impl std::error::Error for DiscoveryError {}

/// Ask a private Agent child process for the current descriptor-only device list.
///
/// 向私有 Agent 子进程请求当前仅基于描述符的设备列表。
pub fn discover_via_agent() -> Result<DeviceList, DiscoveryError> {
    let current_executable = env::current_exe()
        .map_err(|error| DiscoveryError::new("Unable to locate the RazeRS executable.", error))?;
    let (program, arguments) = agent_command(&current_executable);
    request_devices(program, &arguments)
}

fn agent_command(current_executable: &Path) -> (PathBuf, Vec<OsString>) {
    let executable_name = if cfg!(windows) {
        "razers-agent.exe"
    } else {
        "razers-agent"
    };
    let sibling = current_executable.with_file_name(executable_name);
    if sibling.is_file() {
        (sibling, vec!["--stdio".into()])
    } else {
        (
            current_executable.to_path_buf(),
            vec!["--agent-stdio".into()],
        )
    }
}

fn request_devices(program: PathBuf, arguments: &[OsString]) -> Result<DeviceList, DiscoveryError> {
    let mut child = Command::new(&program)
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| DiscoveryError::new("Unable to start the local RazeRS Agent.", error))?;
    let request = Request::new(METHOD_DEVICES_LIST, json!(1));
    let mut input = child
        .stdin
        .take()
        .ok_or_else(|| DiscoveryError::new("Unable to open the local RazeRS Agent input.", ""))?;
    serde_json::to_writer(&mut input, &request)
        .map_err(|error| DiscoveryError::new("Unable to encode the Agent request.", error))?;
    input
        .write_all(b"\n")
        .map_err(|error| DiscoveryError::new("Unable to send the Agent request.", error))?;
    drop(input);

    let output = child.wait_with_output().map_err(|error| {
        DiscoveryError::new("Unable to wait for the local RazeRS Agent.", error)
    })?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(DiscoveryError::new(
            "The local RazeRS Agent stopped unexpectedly.",
            detail,
        ));
    }
    decode_device_response(&output.stdout)
}

fn decode_device_response(encoded: &[u8]) -> Result<DeviceList, DiscoveryError> {
    let response: Response = serde_json::from_slice(encoded).map_err(|error| {
        DiscoveryError::new("The local RazeRS Agent returned invalid data.", error)
    })?;
    if response.jsonrpc != JSON_RPC_VERSION || response.id != json!(1) {
        return Err(DiscoveryError::new(
            "The local RazeRS Agent returned a mismatched response.",
            "",
        ));
    }
    if let Some(error) = response.error {
        let message = match error.code {
            razers_ipc::ERROR_PROTOCOL_VERSION => {
                "The local RazeRS Agent uses an incompatible protocol version."
            }
            razers_ipc::ERROR_INTERNAL => "Device discovery failed.",
            _ => "Agent request failed.",
        };
        return Err(DiscoveryError::new(
            message,
            format!(
                "{} ({}) {}",
                error.message,
                error.code,
                error.data.unwrap_or_default()
            ),
        ));
    }
    match response.result {
        Some(ResponseResult::DeviceList(devices))
            if devices.protocol_version == PROTOCOL_VERSION =>
        {
            Ok(devices)
        }
        Some(ResponseResult::DeviceList(_)) => Err(DiscoveryError::new(
            "The local RazeRS Agent uses an incompatible protocol version.",
            "",
        )),
        Some(ResponseResult::AgentInfo(_)) | None => Err(DiscoveryError::new(
            "The local RazeRS Agent returned an unexpected result.",
            "",
        )),
    }
}

#[cfg(test)]
mod tests {
    use razers_ipc::{DeviceSummary, Response};

    use super::*;

    #[test]
    fn decodes_a_versioned_device_response() {
        let response = Response::success(
            json!(1),
            ResponseResult::DeviceList(DeviceList {
                protocol_version: PROTOCOL_VERSION,
                devices: vec![DeviceSummary {
                    display_name: "Razer Test Mouse".into(),
                    vid: 0x1532,
                    pid: 1,
                    interface_count: 2,
                    vendor_interface_count: 1,
                    support_label: "Detected".into(),
                    support_detail: "Read-only".into(),
                    capabilities: vec!["DPI".into()],
                    evidence_label: "Recorded".into(),
                    evidence_source_count: None,
                    control_available: false,
                }],
                interface_count: 2,
            }),
        );

        let decoded = decode_device_response(&serde_json::to_vec(&response).unwrap()).unwrap();

        assert_eq!(decoded.devices[0].display_name, "Razer Test Mouse");
        assert_eq!(decoded.protocol_version, PROTOCOL_VERSION);
    }

    #[test]
    fn rejects_error_and_wrong_method_responses() {
        let error = Response::failure(json!(1), -32603, "Device discovery failed");
        let wrong_result = Response::success(
            json!(1),
            ResponseResult::AgentInfo(razers_ipc::AgentInfo {
                protocol_version: PROTOCOL_VERSION,
                agent_version: "0.1.0".into(),
                access_mode: "descriptor-only".into(),
                transport: "stdio-child".into(),
            }),
        );

        assert!(decode_device_response(&serde_json::to_vec(&error).unwrap()).is_err());
        assert!(decode_device_response(&serde_json::to_vec(&wrong_result).unwrap()).is_err());
    }

    #[test]
    fn rejects_an_incompatible_result_protocol_version() {
        let response = Response::success(
            json!(1),
            ResponseResult::DeviceList(DeviceList {
                protocol_version: PROTOCOL_VERSION + 1,
                devices: Vec::new(),
                interface_count: 0,
            }),
        );

        let error = decode_device_response(&serde_json::to_vec(&response).unwrap()).unwrap_err();

        assert!(error.message.contains("incompatible protocol version"));
    }

    #[test]
    fn falls_back_to_the_app_when_a_sibling_agent_is_missing() {
        let temporary =
            std::env::temp_dir().join(format!("razers-agent-path-test-{}", std::process::id()));
        let app = temporary.join(if cfg!(windows) {
            "razers.exe"
        } else {
            "razers"
        });

        let (fallback, fallback_arguments) = agent_command(&app);

        assert_eq!(fallback, app);
        assert_eq!(fallback_arguments, [OsString::from("--agent-stdio")]);
    }
}
