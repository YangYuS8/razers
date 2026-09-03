// SPDX-License-Identifier: GPL-2.0-or-later

use std::{collections::BTreeMap, env, path::Path};

use razers_device_registry::{
    DeviceKind, Registry,
    upstream::{
        IrazerCatalog, IrazerDevice, OpenRgbCatalog, OpenRgbDevice, UpstreamCatalog,
        UpstreamDevice, UpstreamFeature,
    },
};
use razers_protocol_core::Report90;
use razers_transport_hidapi::{HidInterfaceSummary, enumerate_razer};
use razers_types::{DeviceId, SupportStatus};

const HELP: &str = r#"razersctl - hardware-free RazeRS developer tools

USAGE:
  razersctl registry validate [DIRECTORY]
  razersctl registry list [DIRECTORY]
  razersctl registry show <DEVICE_ID> [DIRECTORY]
  razersctl upstream validate [OPENRAZER_CATALOG OPENRGB_CATALOG IRAZER_CATALOG]
  razersctl upstream stats [OPENRAZER_CATALOG OPENRGB_CATALOG IRAZER_CATALOG]
  razersctl upstream lookup <VID:PID> [OPENRAZER_CATALOG OPENRGB_CATALOG IRAZER_CATALOG]
  razersctl devices [DIRECTORY]
  razersctl report encode <COMMAND_CLASS> <COMMAND_ID> [ARGUMENT_HEX]
  razersctl report decode <REPORT_HEX>
  razersctl help

Numeric command fields accept decimal or 0x-prefixed hexadecimal values.
Registry commands default to ./devices. This milestone never opens hardware.
Upstream commands default to the OpenRazer, OpenRGB, and iRazer catalogs under
./data/upstream. Catalog entries are evidence, not RazeRS support claims.
"#;

const DEFAULT_OPENRAZER_CATALOG: &str = "data/upstream/openrazer-devices.toml";
const DEFAULT_OPENRGB_CATALOG: &str = "data/upstream/openrgb-devices.toml";
const DEFAULT_IRAZER_CATALOG: &str = "data/upstream/irazer-devices.toml";

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
        [group, command] if group == "upstream" && command == "validate" => upstream_validate(
            Path::new(DEFAULT_OPENRAZER_CATALOG),
            Path::new(DEFAULT_OPENRGB_CATALOG),
            Path::new(DEFAULT_IRAZER_CATALOG),
        ),
        [group, command, openrazer, openrgb, irazer]
            if group == "upstream" && command == "validate" =>
        {
            upstream_validate(Path::new(openrazer), Path::new(openrgb), Path::new(irazer))
        }
        [group, command] if group == "upstream" && command == "stats" => upstream_stats(
            Path::new(DEFAULT_OPENRAZER_CATALOG),
            Path::new(DEFAULT_OPENRGB_CATALOG),
            Path::new(DEFAULT_IRAZER_CATALOG),
        ),
        [group, command, openrazer, openrgb, irazer]
            if group == "upstream" && command == "stats" =>
        {
            upstream_stats(Path::new(openrazer), Path::new(openrgb), Path::new(irazer))
        }
        [group, command, identity] if group == "upstream" && command == "lookup" => {
            upstream_lookup(
                identity,
                Path::new(DEFAULT_OPENRAZER_CATALOG),
                Path::new(DEFAULT_OPENRGB_CATALOG),
                Path::new(DEFAULT_IRAZER_CATALOG),
            )
        }
        [group, command, identity, openrazer, openrgb, irazer]
            if group == "upstream" && command == "lookup" =>
        {
            upstream_lookup(
                identity,
                Path::new(openrazer),
                Path::new(openrgb),
                Path::new(irazer),
            )
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

fn load_upstream_catalog(path: &Path) -> Result<UpstreamCatalog, String> {
    UpstreamCatalog::load_file(path).map_err(|error| error.to_string())
}

fn load_openrgb_catalog(path: &Path) -> Result<OpenRgbCatalog, String> {
    OpenRgbCatalog::load_file(path).map_err(|error| error.to_string())
}

fn load_irazer_catalog(path: &Path) -> Result<IrazerCatalog, String> {
    IrazerCatalog::load_file(path).map_err(|error| error.to_string())
}

fn upstream_validate(
    openrazer_path: &Path,
    openrgb_path: &Path,
    irazer_path: &Path,
) -> Result<(), String> {
    let openrazer = load_upstream_catalog(openrazer_path)?;
    let openrgb = load_openrgb_catalog(openrgb_path)?;
    let irazer = load_irazer_catalog(irazer_path)?;
    println!(
        "validated {} evidence-only device identities from {}@{}",
        openrazer.devices.len(),
        openrazer.source.repository,
        &openrazer.source.commit[..12]
    );
    println!(
        "validated {} evidence-only lighting identities from {}@{}",
        openrgb.devices.len(),
        openrgb.source.repository,
        &openrgb.source.commit[..12]
    );
    println!(
        "validated {} evidence-only cross-platform identities from {}@{}",
        irazer.devices.len(),
        irazer.source.repository,
        &irazer.source.commit[..12]
    );
    print_comparison(&openrazer, &openrgb);
    print_irazer_comparison(&irazer, &openrgb);
    Ok(())
}

fn upstream_stats(
    openrazer_path: &Path,
    openrgb_path: &Path,
    irazer_path: &Path,
) -> Result<(), String> {
    let catalog = load_upstream_catalog(openrazer_path)?;
    let openrgb = load_openrgb_catalog(openrgb_path)?;
    let irazer = load_irazer_catalog(irazer_path)?;
    let mut kinds = BTreeMap::new();
    let mut features = BTreeMap::new();
    for device in &catalog.devices {
        *kinds.entry(device.kind.as_str()).or_insert(0_usize) += 1;
        for feature in &device.upstream_features {
            *features.entry(feature.as_str()).or_insert(0_usize) += 1;
        }
    }

    println!("source: {}", catalog.source.name);
    println!("repository: {}", catalog.source.repository);
    println!("commit: {}", catalog.source.commit);
    println!("devices: {}", catalog.devices.len());
    println!(
        "kinds: {}",
        kinds
            .into_iter()
            .map(|(kind, count)| format!("{kind}={count}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "upstream features: {}",
        features
            .into_iter()
            .map(|(feature, count)| format!("{feature}={count}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let mut matrix_families = BTreeMap::new();
    for device in &openrgb.devices {
        *matrix_families
            .entry(device.matrix_family.as_str())
            .or_insert(0_usize) += 1;
    }
    println!("OpenRGB devices: {}", openrgb.devices.len());
    println!(
        "OpenRGB matrix families: {}",
        matrix_families
            .into_iter()
            .map(|(family, count)| format!("{family}={count}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let upstream_supported = irazer
        .devices
        .iter()
        .filter(|device| device.upstream_support.as_str() == "supported")
        .count();
    println!(
        "iRazer devices: {} (upstream-supported={upstream_supported})",
        irazer.devices.len()
    );
    print_comparison(&catalog, &openrgb);
    print_irazer_comparison(&irazer, &openrgb);
    println!("support status: evidence only; hardware verification required");
    Ok(())
}

fn upstream_lookup(
    identity: &str,
    openrazer_path: &Path,
    openrgb_path: &Path,
    irazer_path: &Path,
) -> Result<(), String> {
    let (vid, pid) = parse_usb_identity(identity)?;
    let openrazer = load_upstream_catalog(openrazer_path)?;
    let openrgb = load_openrgb_catalog(openrgb_path)?;
    let irazer = load_irazer_catalog(irazer_path)?;
    let openrazer_device = openrazer.find_usb(vid, pid);
    let openrgb_device = openrgb.find_usb(vid, pid);
    let irazer_device = irazer.find_usb(vid, pid);
    if openrazer_device.is_none() && openrgb_device.is_none() && irazer_device.is_none() {
        return Err(format!(
            "USB identity {vid:04x}:{pid:04x} is absent from all catalogs"
        ));
    }
    if let Some(device) = openrazer_device {
        println!("[OpenRazer]");
        print_upstream_device(device, &openrazer);
    }
    if let Some(device) = openrgb_device {
        if openrazer_device.is_some() {
            println!();
        }
        println!("[OpenRGB]");
        print_openrgb_device(device, &openrgb);
    }
    if let Some(device) = irazer_device {
        if openrazer_device.is_some() || openrgb_device.is_some() {
            println!();
        }
        println!("[iRazer]");
        print_irazer_device(device, &irazer);
    }
    Ok(())
}

fn print_comparison(openrazer: &UpstreamCatalog, openrgb: &OpenRgbCatalog) {
    let comparison = openrazer.compare_openrgb(openrgb);
    println!(
        "cross-source: overlap={}, OpenRazer-only={}, OpenRGB-only={}, name-differences={}, matrix-differences={}",
        comparison.overlap,
        comparison.openrazer_only,
        comparison.openrgb_only,
        comparison.name_differences,
        comparison.matrix_differences
    );
}

fn print_irazer_comparison(irazer: &IrazerCatalog, openrgb: &OpenRgbCatalog) {
    let irazer_identities = irazer
        .devices
        .iter()
        .map(|device| ((device.vid, device.pid), device))
        .collect::<BTreeMap<_, _>>();
    let openrgb_identities = openrgb
        .devices
        .iter()
        .map(|device| ((device.vid, device.pid), device))
        .collect::<BTreeMap<_, _>>();
    let overlap = irazer_identities
        .keys()
        .filter(|identity| openrgb_identities.contains_key(identity))
        .count();
    let protocol_differences = irazer_identities
        .iter()
        .filter_map(|(identity, irazer_device)| {
            openrgb_identities
                .get(identity)
                .map(|openrgb_device| (*irazer_device, *openrgb_device))
        })
        .filter(|(irazer_device, openrgb_device)| {
            irazer_device.matrix_family != openrgb_device.matrix_family
                || irazer_device.transaction_id != openrgb_device.transaction_id
        })
        .count();

    println!(
        "iRazer/OpenRGB: overlap={}, iRazer-only={}, OpenRGB-only={}, protocol-differences={}",
        overlap,
        irazer_identities.len() - overlap,
        openrgb_identities.len() - overlap,
        protocol_differences
    );
}

fn print_upstream_device(device: &UpstreamDevice, catalog: &UpstreamCatalog) {
    println!("name: {}", device.name);
    println!("kind: {}", device.kind.as_str());
    println!("usb: {:04x}:{:04x}", device.vid, device.pid);
    println!(
        "upstream features: {}",
        feature_names(&device.upstream_features)
    );
    if let Some([rows, columns]) = device.matrix {
        println!("matrix: {rows}x{columns}");
    }
    if let Some(max_dpi) = device.max_dpi {
        println!("max dpi: {max_dpi}");
    }
    if !device.poll_rates_hz.is_empty() {
        println!(
            "poll rates: {} Hz",
            device
                .poll_rates_hz
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!("upstream methods: {}", device.methods.join(", "));
    println!(
        "evidence: {}/{}@{}:{}",
        catalog.source.repository,
        device.source_path,
        &catalog.source.commit[..12],
        device.source_symbol
    );
    println!("support status: evidence only; hardware verification required");
}

fn print_openrgb_device(device: &OpenRgbDevice, catalog: &OpenRgbCatalog) {
    println!("name: {}", device.name);
    println!("kind: {}", device.kind.as_str());
    println!("usb: {:04x}:{:04x}", device.vid, device.pid);
    println!("matrix family: {}", device.matrix_family.as_str());
    println!("transaction id: 0x{:02x}", device.transaction_id);
    println!("matrix: {}x{}", device.matrix[0], device.matrix[1]);
    if !device.zones.is_empty() {
        println!("zone symbols: {}", device.zones.join(", "));
    }
    if let Some(layout) = &device.layout {
        println!("layout symbol: {layout}");
    }
    println!(
        "evidence: {}/{}@{}:{} ({})",
        catalog.source.repository,
        device.source_path,
        &catalog.source.commit[..12],
        device.source_symbol,
        device.pid_symbol
    );
    println!("support status: evidence only; hardware verification required");
}

fn print_irazer_device(device: &IrazerDevice, catalog: &IrazerCatalog) {
    println!("name: {}", device.name);
    println!("kind: {}", device.kind.as_str());
    println!("usb: {:04x}:{:04x}", device.vid, device.pid);
    println!("source id: {}", device.source_id);
    println!(
        "upstream support claim: {}",
        device.upstream_support.as_str()
    );
    println!("capability labels: {}", device.capability_labels.join(", "));
    println!("matrix family: {}", device.matrix_family.as_str());
    println!("transaction id: 0x{:02x}", device.transaction_id);
    println!(
        "evidence: {}/{}@{}:{}",
        catalog.source.repository,
        device.source_path,
        &catalog.source.commit[..12],
        device.source_symbol
    );
    println!("RazeRS support status: evidence only; hardware verification required");
}

fn feature_names(features: &[UpstreamFeature]) -> String {
    features
        .iter()
        .map(|feature| feature.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn devices(directory: &Path) -> Result<(), String> {
    let registry = load_registry(directory)?;
    let openrazer_path = Path::new(DEFAULT_OPENRAZER_CATALOG);
    let openrazer = openrazer_path
        .is_file()
        .then(|| load_upstream_catalog(openrazer_path))
        .transpose()?;
    let openrgb_path = Path::new(DEFAULT_OPENRGB_CATALOG);
    let openrgb = openrgb_path
        .is_file()
        .then(|| load_openrgb_catalog(openrgb_path))
        .transpose()?;
    let irazer_path = Path::new(DEFAULT_IRAZER_CATALOG);
    let irazer = irazer_path
        .is_file()
        .then(|| load_irazer_catalog(irazer_path))
        .transpose()?;
    let interfaces = enumerate_razer().map_err(|error| error.to_string())?;
    if interfaces.is_empty() {
        println!("no Razer HID interfaces detected");
        return Ok(());
    }

    for interface in interfaces {
        print_interface(
            &registry,
            openrazer.as_ref(),
            openrgb.as_ref(),
            irazer.as_ref(),
            &interface,
        );
    }
    Ok(())
}

fn print_interface(
    registry: &Registry,
    openrazer: Option<&UpstreamCatalog>,
    openrgb: Option<&OpenRgbCatalog>,
    irazer: Option<&IrazerCatalog>,
    interface: &HidInterfaceSummary,
) {
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
    let openrazer_match = openrazer
        .and_then(|catalog| catalog.find_usb(interface.vendor_id, interface.product_id))
        .map(|device| device.name.as_str())
        .unwrap_or("unknown");
    let openrgb_match = openrgb
        .and_then(|catalog| catalog.find_usb(interface.vendor_id, interface.product_id))
        .map(|device| device.name.as_str())
        .unwrap_or("unknown");
    let irazer_match = irazer
        .and_then(|catalog| catalog.find_usb(interface.vendor_id, interface.product_id))
        .map(|device| device.name.as_str())
        .unwrap_or("unknown");

    println!(
        "{:04x}:{:04x}\tinterface={}\tusage={:04x}:{:04x}\tcollection={}\taccess=descriptor-only\tproduct={}\tserial={}\tregistry={}\topenrazer={}\topenrgb={}\tirazer={}",
        interface.vendor_id,
        interface.product_id,
        interface.interface_number,
        interface.usage_page,
        interface.usage,
        interface.collection_kind().as_str(),
        interface.product.as_deref().unwrap_or("unknown"),
        if interface.serial_number_present {
            "present-redacted"
        } else {
            "absent"
        },
        registry_match,
        openrazer_match,
        openrgb_match,
        irazer_match
    );
}

fn parse_usb_identity(value: &str) -> Result<(u16, u16), String> {
    let (vid, pid) = value
        .split_once(':')
        .ok_or_else(|| format!("'{value}' must use VID:PID hexadecimal form"))?;
    let parse = |part: &str| {
        u16::from_str_radix(part.strip_prefix("0x").unwrap_or(part), 16)
            .map_err(|_| format!("'{value}' must use VID:PID hexadecimal form"))
    };
    Ok((parse(vid)?, parse(pid)?))
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

    #[test]
    fn parses_usb_identities_as_hexadecimal() {
        assert_eq!(parse_usb_identity("1532:0099").unwrap(), (0x1532, 0x0099));
        assert_eq!(
            parse_usb_identity("0x1532:0x0099").unwrap(),
            (0x1532, 0x0099)
        );
        assert!(parse_usb_identity("1532").is_err());
    }
}
