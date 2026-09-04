// SPDX-License-Identifier: GPL-2.0-or-later
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn language_selection_never_changes_json_rpc() {
    let request = b"{\"jsonrpc\":\"2.0\",\"method\":\"agent.info\",\"params\":{\"protocol_version\":1},\"id\":1}\n";
    let replies: Vec<_> = ["en", "zh-CN"]
        .into_iter()
        .map(|language| {
            let mut child = Command::new(env!("CARGO_BIN_EXE_razers-agent"))
                .args(["--lang", language, "--stdio"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            child.stdin.take().unwrap().write_all(request).unwrap();
            let output = child.wait_with_output().unwrap();
            assert!(output.status.success());
            assert!(output.stderr.is_empty());
            output.stdout
        })
        .collect();
    assert_eq!(replies[0], replies[1]);
    let value: serde_json::Value = serde_json::from_slice(&replies[0]).unwrap();
    assert_eq!(value["result"]["access_mode"], "descriptor-only");
}
