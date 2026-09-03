// SPDX-License-Identifier: GPL-2.0-or-later

use std::{env, path::Path};

use razers_device_registry::{DeviceKind, Registry};
use razers_protocol_core::Report90;
use razers_transport_hidapi::{HidInterfaceSummary, enumerate_razer};
use razers_types::{DeviceId, SupportStatus};

const HELP: &str = r#"razersctl - hardware-free Razers developer tools

USAGE:
  razersctl registry validate [DIRECTORY]
  razersctl registry list [DIRECTORY]
  razersctl registry show <DEVICE_ID> [DIRECTORY]
  razersctl devices [DIRECTORY]
  razersctl report encode <COMMAND_CLASS> <COMMAND_ID> [ARGUMENT_HEX]
  razersctl report decode <REPORT_HEX>
  razersctl help

Numeric command fields accept decimal or 0x-prefixed hexadecimal values.
Registry commands default to ./devices. This milestone never opens hardware.
"#;

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        eprintln!("error: {error}");
        std::process::exit(2);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    match args.as_slice() {
        [] => {
            print!("{HELP}");
            Ok(())
        }
        [single] if single == "help" || single == "--help" || single == "-h" => {
            print!("{HELP}");
            Ok(())
        }
        [single] if single == "--version" || single == "-V" => {
            println!("razersctl {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        [group, command] if group == "registry" && command == "validate" => {
            registry_validate(Path::new("devices"))
        }
        [group, command, directory] if group == "registry" && command == "validate" => {
            registry_validate(Path::new(directory))
        }
        [group, command] if group == "registry" && command == "list" => {
            registry_list(Path::new("devices"))
        }
        [group, command, directory] if group == "registry" && command == "list" => {
            registry_list(Path::new(directory))
        }
        [group, command, id] if group == "registry" && command == "show" => {
            registry_show(id, Path::new("devices"))
        }
        [group, command, id, directory] if group == "registry" && command == "show" => {
            registry_show(id, Path::new(directory))
        }
        [command] if command == "devices" => devices(Path::new("devices")),
        [command, directory] if command == "devices" => devices(Path::new(directory)),
        [group, command, class, id] if group == "report" && command == "encode" => {
            report_encode(class, id, "")
        }
        [group, command, class, id, arguments] if group == "report" && command == "encode" => {
            report_encode(class, id, arguments)
        }
        [group, command, report] if group == "report" && command == "decode" => {
            report_decode(report)
        }
        _ => Err(format!("unrecognized arguments\n\n{HELP}")),
    }
}

fn load_registry(directory: &Path) -> Result<Registry, String> {
    let registry = Registry::load_dir(directory).map_err(|error| error.to_string())?;
    if registry.is_empty() {
        return Err(format!(
            "registry directory '{}' contains no TOML manifests",
            directory.display()
        ));
    }
    Ok(registry)
}

fn registry_validate(directory: &Path) -> Result<(), String> {
    let registry = load_registry(directory)?;
    println!(
        "validated {} device manifest{} in {}",
        registry.len(),
        if registry.len() == 1 { "" } else { "s" },
        directory.display()
    );
    Ok(())
}

fn registry_list(directory: &Path) -> Result<(), String> {
    let registry = load_registry(directory)?;
    for loaded in registry.iter() {
        let device = &loaded.descriptor;
        println!(
            "{}\t{}\t{}\t{}\t{} capabilities",
            device.id,
            device.display_name,
            device_kind(device.kind),
            support_status(device.support.status),
            device.capabilities.count()
        );
    }
    Ok(())
}

fn registry_show(id: &str, directory: &Path) -> Result<(), String> {
    let id = DeviceId::new(id).map_err(|error| error.to_string())?;
    let registry = load_registry(directory)?;
    let loaded = registry
        .get(&id)
        .ok_or_else(|| format!("device '{id}' was not found in {}", directory.display()))?;
    let device = &loaded.descriptor;

    println!("id: {}", device.id);
    println!("name: {}", device.display_name);
    println!("kind: {}", device_kind(device.kind));
    println!("support: {}", support_status(device.support.status));
    println!("notes: {}", device.support.notes);
    println!("manifest: {}", loaded.source_path.display());
    println!("connections:");
    for connection in &device.connections {
        println!(
            "  - {}: {:?}, {:04x}:{:04x}, protocol={}, report-id=0x{:02x}",
            connection.id,
            connection.transport,
            connection.identity.vid,
            connection.identity.pid,
            connection.protocol.family,
            connection.protocol.report_id
        );
    }
    println!(
        "capabilities: {}",
        device.capabilities.names().collect::<Vec<_>>().join(", ")
    );
    println!("evidence:");
    for evidence in &device.evidence {
        println!(
            "  - {}/{}@{}:{} ({})",
            evidence.repository,
            evidence.path,
            &evidence.commit[..12],
            evidence.symbol,
            evidence.license
        );
    }
    Ok(())
}

fn devices(directory: &Path) -> Result<(), String> {
    let registry = load_registry(directory)?;
    let interfaces = enumerate_razer().map_err(|error| error.to_string())?;
    if interfaces.is_empty() {
        println!("no Razer HID interfaces detected");
        return Ok(());
    }

    for interface in interfaces {
        print_interface(&registry, &interface);
    }
    Ok(())
}

fn print_interface(registry: &Registry, interface: &HidInterfaceSummary) {
    let matches = registry
        .iter()
        .filter(|loaded| {
            loaded.descriptor.connections.iter().any(|connection| {
                connection.identity.matches_hid(
                    interface.vendor_id,
                    interface.product_id,
                    interface.usage_page,
                    interface.usage,
                    interface.interface_number,
                )
            })
        })
        .map(|loaded| loaded.descriptor.id.as_str())
        .collect::<Vec<_>>();
    let registry_match = if matches.is_empty() {
        "unknown".to_owned()
    } else {
        matches.join(",")
    };

    println!(
        "{:04x}:{:04x}\tinterface={}\tusage={:04x}:{:04x}\tproduct={}\tserial={}\tregistry={}",
        interface.vendor_id,
        interface.product_id,
        interface.interface_number,
        interface.usage_page,
        interface.usage,
        interface.product.as_deref().unwrap_or("unknown"),
        if interface.serial_number_present {
            "present-redacted"
        } else {
            "absent"
        },
        registry_match
    );
}

fn report_encode(class: &str, id: &str, arguments: &str) -> Result<(), String> {
    let class = parse_byte(class)?;
    let id = parse_byte(id)?;
    let arguments = parse_hex(arguments)?;
    let report = Report90::command(class, id, arguments).map_err(|error| error.to_string())?;
    let bytes = report.encode().map_err(|error| error.to_string())?;
    println!("{}", format_hex(&bytes));
    Ok(())
}

fn report_decode(encoded: &str) -> Result<(), String> {
    let bytes = parse_hex(encoded)?;
    let report = Report90::decode(&bytes).map_err(|error| error.to_string())?;

    println!("status: 0x{:02x}", report.status);
    println!("transaction-id: 0x{:02x}", report.transaction_id);
    println!("remaining-packets: {}", report.remaining_packets);
    println!("protocol-type: 0x{:02x}", report.protocol_type);
    println!("command-class: 0x{:02x}", report.command_class);
    println!("command-id: 0x{:02x}", report.command_id);
    println!("arguments: {}", format_hex(report.arguments()));
    println!("reserved: 0x{:02x}", report.reserved);
    Ok(())
}

fn parse_byte(value: &str) -> Result<u8, String> {
    let parsed = if let Some(hex) = value.strip_prefix("0x") {
        u8::from_str_radix(hex, 16)
    } else {
        value.parse()
    };
    parsed.map_err(|_| format!("'{value}' is not a byte value"))
}

fn parse_hex(value: &str) -> Result<Vec<u8>, String> {
    let compact = value
        .chars()
        .filter(|character| !character.is_ascii_whitespace() && !matches!(character, ':' | '-'))
        .collect::<String>();
    let compact = compact.strip_prefix("0x").unwrap_or(&compact);
    if compact.len() % 2 != 0 {
        return Err("hex input must contain an even number of digits".into());
    }

    (0..compact.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&compact[index..index + 2], 16).map_err(|_| {
                format!(
                    "invalid hex byte '{}': position {index}",
                    &compact[index..index + 2]
                )
            })
        })
        .collect()
}

fn format_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    bytes.iter().fold(String::new(), |mut encoded, byte| {
        write!(encoded, "{byte:02x}").expect("writing to a String cannot fail");
        encoded
    })
}

fn support_status(status: SupportStatus) -> &'static str {
    match status {
        SupportStatus::Detected => "detected",
        SupportStatus::Experimental => "experimental",
        SupportStatus::Verified => "verified",
        SupportStatus::Regressed => "regressed",
        SupportStatus::Unsupported => "unsupported",
    }
}

fn device_kind(kind: DeviceKind) -> &'static str {
    match kind {
        DeviceKind::Mouse => "mouse",
        DeviceKind::Keyboard => "keyboard",
        DeviceKind::Headset => "headset",
        DeviceKind::Speaker => "speaker",
        DeviceKind::MouseMat => "mouse-mat",
        DeviceKind::Laptop => "laptop",
        DeviceKind::Receiver => "receiver",
        DeviceKind::Accessory => "accessory",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_human_friendly_hex() {
        assert_eq!(parse_hex("0x00:aa-bb cc").unwrap(), [0, 0xaa, 0xbb, 0xcc]);
    }

    #[test]
    fn rejects_partial_hex_bytes() {
        assert!(parse_hex("abc").is_err());
    }
}
