// SPDX-License-Identifier: GPL-2.0-or-later
use std::process::{Command, Output};

fn run(arguments: &[&str], environment: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_razersctl"));
    command
        .args(arguments)
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    for name in ["RAZERS_LANG", "LC_ALL", "LC_MESSAGES", "LANG"] {
        command.env_remove(name);
    }
    command.envs(environment.iter().copied()).output().unwrap()
}

#[test]
fn cli_override_and_environment_precedence() {
    for (args, env, expected) in [
        (
            vec!["--lang", "en", "--help"],
            vec![("RAZERS_LANG", "zh-CN")],
            "USAGE",
        ),
        (
            vec!["--help", "--lang=zh-CN"],
            vec![("LC_ALL", "C")],
            "用法",
        ),
        (
            vec!["--help"],
            vec![("RAZERS_LANG", "zh-CN"), ("LC_ALL", "en_US.UTF-8")],
            "用法",
        ),
        (
            vec!["--help"],
            vec![("LC_ALL", "en_US.UTF-8"), ("LANG", "zh_CN.UTF-8")],
            "USAGE",
        ),
        (
            vec!["--help"],
            vec![("RAZERS_LANG", "auto"), ("LC_MESSAGES", "zh_CN.UTF-8")],
            "用法",
        ),
        (
            vec!["--lang=auto", "--help"],
            vec![("LANG", "fr_FR.UTF-8")],
            "USAGE",
        ),
    ] {
        let output = run(&args, &env);
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains(expected), "{args:?} {env:?}: {text}");
    }
}

#[test]
fn binary_reports_and_identifiers_are_locale_independent() {
    let en = run(
        &["--lang=en", "report", "encode", "0x00", "0x81", "0000"],
        &[],
    );
    let zh = run(
        &["--lang=zh-CN", "report", "encode", "0x00", "0x81", "0000"],
        &[],
    );
    assert!(en.status.success() && zh.status.success());
    assert_eq!(en.stdout, zh.stdout);
    assert_eq!(en.stdout.len(), 181);
    let output = run(
        &["--lang=zh-CN", "registry", "show", "razer.basilisk-v3"],
        &[],
    );
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("标识：razer.basilisk-v3"));
    assert!(text.contains("1532:0099"));
}

#[test]
fn chinese_errors_handle_unicode_input_without_panicking() {
    for args in [
        vec!["--lang=zh-CN", "report", "decode", "鼠标"],
        vec!["--lang=zh-CN", "invalid"],
    ] {
        let output = run(&args, &[]);
        assert_eq!(output.status.code(), Some(2));
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains("错误"));
        assert!(!error.contains("panicked"));
    }
    let missing = run(&["--lang"], &[("LANG", "zh_CN.UTF-8")]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(
        String::from_utf8(missing.stderr)
            .unwrap()
            .contains("需要指定")
    );
}
