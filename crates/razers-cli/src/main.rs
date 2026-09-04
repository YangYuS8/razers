// SPDX-License-Identifier: GPL-2.0-or-later

use razers_i18n::{Locale, language_args};
use std::{collections::BTreeMap, env, path::Path, sync::OnceLock};

static LOCALE: OnceLock<Locale> = OnceLock::new();
fn locale() -> Locale {
    LOCALE.get().copied().unwrap_or_default()
}

use razers_device_registry::{
    DeviceKind, Registry,
    upstream::{
        EvidenceAssessment, EvidenceReadiness, IrazerCatalog, IrazerDevice, OpenRgbCatalog,
        OpenRgbDevice, UpstreamCatalog, UpstreamDevice, UpstreamFeature, assess_evidence,
    },
};
use razers_protocol_core::Report90;
use razers_transport_hidapi::{HidInterfaceSummary, enumerate_razer};
use razers_types::{DeviceId, SupportStatus};

const DEFAULT_OPENRAZER_CATALOG: &str = "data/upstream/openrazer-devices.toml";
const DEFAULT_OPENRGB_CATALOG: &str = "data/upstream/openrgb-devices.toml";
const DEFAULT_IRAZER_CATALOG: &str = "data/upstream/irazer-devices.toml";

fn main() {
    let parsed = language_args(env::args().skip(1).collect());
    let language = parsed
        .as_ref()
        .ok()
        .and_then(|(language, _)| *language)
        .map_or_else(Locale::system, |value| value.resolve());
    let _ = LOCALE.set(language);
    if let Err(error) = parsed.and_then(|(_, args)| run(args)) {
        eprintln!(
            "{}: {}\n{}",
            locale().text("error"),
            locale().text("Operation failed; diagnostic detail"),
            locale().text(&error)
        );
        std::process::exit(2);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    match args.as_slice() {
        [] => {
            print!("{}", locale().text("cli.help"));
            Ok(())
        }
        [single] if single == "help" || single == "--help" || single == "-h" => {
            print!("{}", locale().text("cli.help"));
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
        [group, command, identity] if group == "upstream" && command == "assess" => {
            upstream_assess(
                identity,
                Path::new(DEFAULT_OPENRAZER_CATALOG),
                Path::new(DEFAULT_OPENRGB_CATALOG),
                Path::new(DEFAULT_IRAZER_CATALOG),
            )
        }
        [group, command, identity, openrazer, openrgb, irazer]
            if group == "upstream" && command == "assess" =>
        {
            upstream_assess(
                identity,
                Path::new(openrazer),
                Path::new(openrgb),
                Path::new(irazer),
            )
        }
        [group, command] if group == "upstream" && command == "shortlist" => {
            upstream_assessment_list(
                EvidenceReadiness::Corroborated,
                Path::new(DEFAULT_OPENRAZER_CATALOG),
                Path::new(DEFAULT_OPENRGB_CATALOG),
                Path::new(DEFAULT_IRAZER_CATALOG),
            )
        }
        [group, command, openrazer, openrgb, irazer]
            if group == "upstream" && command == "shortlist" =>
        {
            upstream_assessment_list(
                EvidenceReadiness::Corroborated,
                Path::new(openrazer),
                Path::new(openrgb),
                Path::new(irazer),
            )
        }
        [group, command] if group == "upstream" && command == "conflicts" => {
            upstream_assessment_list(
                EvidenceReadiness::NeedsResearch,
                Path::new(DEFAULT_OPENRAZER_CATALOG),
                Path::new(DEFAULT_OPENRGB_CATALOG),
                Path::new(DEFAULT_IRAZER_CATALOG),
            )
        }
        [group, command, openrazer, openrgb, irazer]
            if group == "upstream" && command == "conflicts" =>
        {
            upstream_assessment_list(
                EvidenceReadiness::NeedsResearch,
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
        _ => Err(locale().format(
            "unrecognized arguments\n\n{HELP}",
            &[locale().text("cli.help").to_string()],
        )),
    }
}

fn load_registry(directory: &Path) -> Result<Registry, String> {
    let registry = Registry::load_dir(directory).map_err(|error| error.to_string())?;
    if registry.is_empty() {
        return Err(locale().format(
            "registry directory '{}' contains no TOML manifests",
            &[format!("{}", directory.display())],
        ));
    }
    Ok(registry)
}

fn registry_validate(directory: &Path) -> Result<(), String> {
    let registry = load_registry(directory)?;
    println!(
        "{}",
        locale().format(
            "device manifests validated: {} in {}",
            &[
                format!("{}", registry.len()),
                format!("{}", directory.display())
            ]
        )
    );
    Ok(())
}

fn registry_list(directory: &Path) -> Result<(), String> {
    let registry = load_registry(directory)?;
    for loaded in registry.iter() {
        let device = &loaded.descriptor;
        println!(
            "{}",
            locale().format(
                "{}\t{}\t{}\t{}\t{} capabilities",
                &[
                    format!("{}", device.id),
                    device.display_name.to_string(),
                    device_kind(device.kind).to_string(),
                    support_status(device.support.status).to_string(),
                    format!("{}", device.capabilities.count())
                ]
            )
        );
    }
    Ok(())
}

fn registry_show(id: &str, directory: &Path) -> Result<(), String> {
    let id = DeviceId::new(id).map_err(|error| error.to_string())?;
    let registry = load_registry(directory)?;
    let loaded = registry.get(&id).ok_or_else(|| {
        locale().format(
            "device '{id}' was not found in {}",
            &[format!("{}", id), format!("{}", directory.display())],
        )
    })?;
    let device = &loaded.descriptor;

    println!("{}", locale().format("id: {}", &[format!("{}", device.id)]));
    println!(
        "{}",
        locale().format("name: {}", std::slice::from_ref(&device.display_name))
    );
    println!(
        "{}",
        locale().format("kind: {}", &[device_kind(device.kind).to_string()])
    );
    println!(
        "{}",
        locale().format(
            "support: {}",
            &[support_status(device.support.status).to_string()]
        )
    );
    println!(
        "{}",
        locale().format("notes: {}", std::slice::from_ref(&device.support.notes))
    );
    println!(
        "{}",
        locale().format(
            "manifest: {}",
            &[format!("{}", loaded.source_path.display())]
        )
    );
    println!("{}", locale().format("connections:", &[]));
    for connection in &device.connections {
        println!(
            "{}",
            locale().format(
                "  - {}: {:?}, {:04x}:{:04x}, protocol={}, report-id=0x{:02x}",
                &[
                    format!("{}", connection.id),
                    format!("{:?}", connection.transport),
                    format!("{:04x}", connection.identity.vid),
                    format!("{:04x}", connection.identity.pid),
                    connection.protocol.family.to_string(),
                    format!("{:02x}", connection.protocol.report_id)
                ]
            )
        );
    }
    println!(
        "{}",
        locale().format(
            "capabilities: {}",
            &[device
                .capabilities
                .names()
                .collect::<Vec<_>>()
                .join(", ")
                .to_string()]
        )
    );
    println!("{}", locale().format("evidence:", &[]));
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
        "{}",
        locale().format(
            "validated {} evidence-only device identities from {}@{}",
            &[
                format!("{}", openrazer.devices.len()),
                openrazer.source.repository.to_string(),
                openrazer.source.commit[..12].to_string()
            ]
        )
    );
    println!(
        "{}",
        locale().format(
            "validated {} evidence-only lighting identities from {}@{}",
            &[
                format!("{}", openrgb.devices.len()),
                openrgb.source.repository.to_string(),
                openrgb.source.commit[..12].to_string()
            ]
        )
    );
    println!(
        "{}",
        locale().format(
            "validated {} evidence-only cross-platform identities from {}@{}",
            &[
                format!("{}", irazer.devices.len()),
                irazer.source.repository.to_string(),
                irazer.source.commit[..12].to_string()
            ]
        )
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

    println!(
        "{}",
        locale().format("source: {}", std::slice::from_ref(&catalog.source.name))
    );
    println!(
        "{}",
        locale().format(
            "repository: {}",
            std::slice::from_ref(&catalog.source.repository)
        )
    );
    println!(
        "{}",
        locale().format("commit: {}", std::slice::from_ref(&catalog.source.commit))
    );
    println!(
        "{}",
        locale().format("devices: {}", &[format!("{}", catalog.devices.len())])
    );
    println!(
        "{}",
        locale().format(
            "kinds: {}",
            &[kinds
                .into_iter()
                .map(|(kind, count)| format!("{kind}={count}"))
                .collect::<Vec<_>>()
                .join(", ")
                .to_string()]
        )
    );
    println!(
        "{}",
        locale().format(
            "upstream features: {}",
            &[features
                .into_iter()
                .map(|(feature, count)| format!("{feature}={count}"))
                .collect::<Vec<_>>()
                .join(", ")
                .to_string()]
        )
    );
    let mut matrix_families = BTreeMap::new();
    for device in &openrgb.devices {
        *matrix_families
            .entry(device.matrix_family.as_str())
            .or_insert(0_usize) += 1;
    }
    println!(
        "{}",
        locale().format(
            "OpenRGB devices: {}",
            &[format!("{}", openrgb.devices.len())]
        )
    );
    println!(
        "{}",
        locale().format(
            "OpenRGB matrix families: {}",
            &[matrix_families
                .into_iter()
                .map(|(family, count)| format!("{family}={count}"))
                .collect::<Vec<_>>()
                .join(", ")
                .to_string()]
        )
    );
    let upstream_supported = irazer
        .devices
        .iter()
        .filter(|device| device.upstream_support.as_str() == "supported")
        .count();
    println!(
        "{}",
        locale().format(
            "iRazer devices: {} (upstream-supported={upstream_supported})",
            &[
                format!("{}", irazer.devices.len()),
                format!("{}", upstream_supported)
            ]
        )
    );
    print_comparison(&catalog, &openrgb);
    print_irazer_comparison(&irazer, &openrgb);
    println!(
        "{}",
        locale().format(
            "support basis: reusable upstream evidence; reconcile before enablement",
            &[]
        )
    );
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
        return Err(locale().format(
            "USB identity {vid:04x}:{pid:04x} is absent from all catalogs",
            &[format!("{:04x}", vid), format!("{:04x}", pid)],
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

fn upstream_assess(
    identity: &str,
    openrazer_path: &Path,
    openrgb_path: &Path,
    irazer_path: &Path,
) -> Result<(), String> {
    let (vid, pid) = parse_usb_identity(identity)?;
    let assessments = load_evidence_assessments(openrazer_path, openrgb_path, irazer_path)?;
    let assessment = assessments
        .iter()
        .find(|assessment| assessment.vid == vid && assessment.pid == pid)
        .ok_or_else(|| {
            locale().format(
                "USB identity {vid:04x}:{pid:04x} is absent from all catalogs",
                &[format!("{:04x}", vid), format!("{:04x}", pid)],
            )
        })?;

    println!("{}", format_evidence_assessment(assessment));
    println!(
        "{}",
        locale().format(
            "note: readiness is a research triage result, not a RazeRS support claim",
            &[]
        )
    );
    Ok(())
}

fn upstream_assessment_list(
    readiness: EvidenceReadiness,
    openrazer_path: &Path,
    openrgb_path: &Path,
    irazer_path: &Path,
) -> Result<(), String> {
    let assessments = load_evidence_assessments(openrazer_path, openrgb_path, irazer_path)?;
    let corroborated = count_readiness(&assessments, EvidenceReadiness::Corroborated);
    let needs_research = count_readiness(&assessments, EvidenceReadiness::NeedsResearch);
    let single_source = count_readiness(&assessments, EvidenceReadiness::SingleSource);
    println!(
        "{}",
        locale().format(
            "assessment: total={}, corroborated={}, needs-research={}, single-source={}",
            &[
                format!("{}", assessments.len()),
                format!("{}", corroborated),
                format!("{}", needs_research),
                format!("{}", single_source)
            ]
        )
    );
    for assessment in assessments
        .iter()
        .filter(|assessment| assessment.readiness == readiness)
    {
        println!("{}", format_evidence_assessment(assessment));
    }
    println!(
        "{}",
        locale().format(
            "note: readiness is a research triage result, not a RazeRS support claim",
            &[]
        )
    );
    Ok(())
}

fn load_evidence_assessments(
    openrazer_path: &Path,
    openrgb_path: &Path,
    irazer_path: &Path,
) -> Result<Vec<EvidenceAssessment>, String> {
    let openrazer = load_upstream_catalog(openrazer_path)?;
    let openrgb = load_openrgb_catalog(openrgb_path)?;
    let irazer = load_irazer_catalog(irazer_path)?;
    Ok(assess_evidence(&openrazer, &openrgb, &irazer))
}

fn count_readiness(assessments: &[EvidenceAssessment], readiness: EvidenceReadiness) -> usize {
    assessments
        .iter()
        .filter(|assessment| assessment.readiness == readiness)
        .count()
}

fn format_evidence_assessment(assessment: &EvidenceAssessment) -> String {
    let sources = assessment
        .sources
        .iter()
        .map(|source| source.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let features = assessment
        .openrazer_features
        .iter()
        .map(|feature| feature.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let features = if features.is_empty() {
        "none"
    } else {
        features.as_str()
    };
    let irazer_support = assessment
        .irazer_support
        .map(|support| support.as_str())
        .unwrap_or("absent");

    locale().format("{:04x}:{:04x}\treadiness={}\tsources={}\tname={}\tkind={}\tmatrix={}\tprotocol={}\topenrazer-features={}\tirazer-support={}", &[format!("{:04x}", assessment.vid), format!("{:04x}", assessment.pid), assessment.readiness.as_str().to_string(), sources.to_string(), assessment.name_agreement.as_str().to_string(), assessment.kind_agreement.as_str().to_string(), assessment.matrix_agreement.as_str().to_string(), assessment.protocol_agreement.as_str().to_string(), features.to_string(), irazer_support.to_string()])
}

fn print_comparison(openrazer: &UpstreamCatalog, openrgb: &OpenRgbCatalog) {
    let comparison = openrazer.compare_openrgb(openrgb);
    println!("{}", locale().format("cross-source: overlap={}, OpenRazer-only={}, OpenRGB-only={}, name-differences={}, matrix-differences={}", &[format!("{}", comparison.overlap), format!("{}", comparison.openrazer_only), format!("{}", comparison.openrgb_only), format!("{}", comparison.name_differences), format!("{}", comparison.matrix_differences)]));
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
        "{}",
        locale().format(
            "iRazer/OpenRGB: overlap={}, iRazer-only={}, OpenRGB-only={}, protocol-differences={}",
            &[
                format!("{}", overlap),
                format!("{}", irazer_identities.len() - overlap),
                format!("{}", openrgb_identities.len() - overlap),
                format!("{}", protocol_differences)
            ]
        )
    );
}

fn print_upstream_device(device: &UpstreamDevice, catalog: &UpstreamCatalog) {
    println!(
        "{}",
        locale().format("name: {}", std::slice::from_ref(&device.name))
    );
    println!(
        "{}",
        locale().format("kind: {}", &[device.kind.as_str().to_string()])
    );
    println!(
        "{}",
        locale().format(
            "usb: {:04x}:{:04x}",
            &[format!("{:04x}", device.vid), format!("{:04x}", device.pid)]
        )
    );
    println!(
        "{}",
        locale().format(
            "upstream features: {}",
            &[feature_names(&device.upstream_features).to_string()]
        )
    );
    if let Some([rows, columns]) = device.matrix {
        println!(
            "{}",
            locale().format(
                "matrix: {rows}x{columns}",
                &[format!("{}", rows), format!("{}", columns)]
            )
        );
    }
    if let Some(max_dpi) = device.max_dpi {
        println!(
            "{}",
            locale().format("max dpi: {max_dpi}", &[format!("{}", max_dpi)])
        );
    }
    if !device.poll_rates_hz.is_empty() {
        println!(
            "{}",
            locale().format(
                "poll rates: {} Hz",
                &[device
                    .poll_rates_hz
                    .iter()
                    .map(u32::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
                    .to_string()]
            )
        );
    }
    println!(
        "{}",
        locale().format(
            "upstream methods: {}",
            &[device.methods.join(", ").to_string()]
        )
    );
    println!(
        "{}",
        locale().format(
            "evidence: {}/{}@{}:{}",
            &[
                catalog.source.repository.to_string(),
                device.source_path.to_string(),
                catalog.source.commit[..12].to_string(),
                device.source_symbol.to_string()
            ]
        )
    );
    println!(
        "{}",
        locale().format(
            "RazeRS status: upstream evidence; apply the evidence policy before enablement",
            &[]
        )
    );
}

fn print_openrgb_device(device: &OpenRgbDevice, catalog: &OpenRgbCatalog) {
    println!(
        "{}",
        locale().format("name: {}", std::slice::from_ref(&device.name))
    );
    println!(
        "{}",
        locale().format("kind: {}", &[device.kind.as_str().to_string()])
    );
    println!(
        "{}",
        locale().format(
            "usb: {:04x}:{:04x}",
            &[format!("{:04x}", device.vid), format!("{:04x}", device.pid)]
        )
    );
    println!(
        "{}",
        locale().format(
            "matrix family: {}",
            &[device.matrix_family.as_str().to_string()]
        )
    );
    println!(
        "{}",
        locale().format(
            "transaction id: 0x{:02x}",
            &[format!("{:02x}", device.transaction_id)]
        )
    );
    println!(
        "{}",
        locale().format(
            "matrix: {}x{}",
            &[
                format!("{}", device.matrix[0]),
                format!("{}", device.matrix[1])
            ]
        )
    );
    if !device.zones.is_empty() {
        println!(
            "{}",
            locale().format("zone symbols: {}", &[device.zones.join(", ").to_string()])
        );
    }
    if let Some(layout) = &device.layout {
        println!(
            "{}",
            locale().format("layout symbol: {layout}", std::slice::from_ref(layout))
        );
    }
    println!(
        "{}",
        locale().format(
            "evidence: {}/{}@{}:{} ({})",
            &[
                catalog.source.repository.to_string(),
                device.source_path.to_string(),
                catalog.source.commit[..12].to_string(),
                device.source_symbol.to_string(),
                device.pid_symbol.to_string()
            ]
        )
    );
    println!(
        "{}",
        locale().format(
            "RazeRS status: upstream evidence; apply the evidence policy before enablement",
            &[]
        )
    );
}

fn print_irazer_device(device: &IrazerDevice, catalog: &IrazerCatalog) {
    println!(
        "{}",
        locale().format("name: {}", std::slice::from_ref(&device.name))
    );
    println!(
        "{}",
        locale().format("kind: {}", &[device.kind.as_str().to_string()])
    );
    println!(
        "{}",
        locale().format(
            "usb: {:04x}:{:04x}",
            &[format!("{:04x}", device.vid), format!("{:04x}", device.pid)]
        )
    );
    println!(
        "{}",
        locale().format("source id: {}", std::slice::from_ref(&device.source_id))
    );
    println!(
        "{}",
        locale().format(
            "upstream support claim: {}",
            &[device.upstream_support.as_str().to_string()]
        )
    );
    println!(
        "{}",
        locale().format(
            "capability labels: {}",
            &[device.capability_labels.join(", ").to_string()]
        )
    );
    println!(
        "{}",
        locale().format(
            "matrix family: {}",
            &[device.matrix_family.as_str().to_string()]
        )
    );
    println!(
        "{}",
        locale().format(
            "transaction id: 0x{:02x}",
            &[format!("{:02x}", device.transaction_id)]
        )
    );
    println!(
        "{}",
        locale().format(
            "evidence: {}/{}@{}:{}",
            &[
                catalog.source.repository.to_string(),
                device.source_path.to_string(),
                catalog.source.commit[..12].to_string(),
                device.source_symbol.to_string()
            ]
        )
    );
    println!(
        "{}",
        locale().format(
            "RazeRS status: upstream evidence; apply the evidence policy before enablement",
            &[]
        )
    );
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
        println!(
            "{}",
            locale().format("no Razer HID interfaces detected", &[])
        );
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

    println!("{}", locale().format("{:04x}:{:04x}\tinterface={}\tusage={:04x}:{:04x}\tcollection={}\taccess=descriptor-only\tproduct={}\tserial={}\tregistry={}\topenrazer={}\topenrgb={}\tirazer={}", &[format!("{:04x}", interface.vendor_id), format!("{:04x}", interface.product_id), format!("{}", interface.interface_number), format!("{:04x}", interface.usage_page), format!("{:04x}", interface.usage), interface.collection_kind().as_str().to_string(), interface.product.as_deref().unwrap_or("unknown").to_string(), (if interface.serial_number_present {
            "present-redacted"
        } else {
            "absent"
        }).to_string(), registry_match.to_string(), openrazer_match.to_string(), openrgb_match.to_string(), irazer_match.to_string()]));
}

fn parse_usb_identity(value: &str) -> Result<(u16, u16), String> {
    let (vid, pid) = value.split_once(':').ok_or_else(|| {
        locale().format(
            "'{value}' must use VID:PID hexadecimal form",
            &[value.to_string()],
        )
    })?;
    let parse = |part: &str| {
        u16::from_str_radix(part.strip_prefix("0x").unwrap_or(part), 16).map_err(|_| {
            locale().format(
                "'{value}' must use VID:PID hexadecimal form",
                &[value.to_string()],
            )
        })
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

    println!(
        "{}",
        locale().format("status: 0x{:02x}", &[format!("{:02x}", report.status)])
    );
    println!(
        "{}",
        locale().format(
            "transaction-id: 0x{:02x}",
            &[format!("{:02x}", report.transaction_id)]
        )
    );
    println!(
        "{}",
        locale().format(
            "remaining-packets: {}",
            &[format!("{}", report.remaining_packets)]
        )
    );
    println!(
        "{}",
        locale().format(
            "protocol-type: 0x{:02x}",
            &[format!("{:02x}", report.protocol_type)]
        )
    );
    println!(
        "{}",
        locale().format(
            "command-class: 0x{:02x}",
            &[format!("{:02x}", report.command_class)]
        )
    );
    println!(
        "{}",
        locale().format(
            "command-id: 0x{:02x}",
            &[format!("{:02x}", report.command_id)]
        )
    );
    println!(
        "{}",
        locale().format(
            "arguments: {}",
            &[format_hex(report.arguments()).to_string()]
        )
    );
    println!(
        "{}",
        locale().format("reserved: 0x{:02x}", &[format!("{:02x}", report.reserved)])
    );
    Ok(())
}

fn parse_byte(value: &str) -> Result<u8, String> {
    let parsed = if let Some(hex) = value.strip_prefix("0x") {
        u8::from_str_radix(hex, 16)
    } else {
        value.parse()
    };
    parsed.map_err(|_| locale().format("'{value}' is not a byte value", &[value.to_string()]))
}

fn parse_hex(value: &str) -> Result<Vec<u8>, String> {
    let compact = value
        .chars()
        .filter(|character| !character.is_ascii_whitespace() && !matches!(character, ':' | '-'))
        .collect::<String>();
    let compact = compact.strip_prefix("0x").unwrap_or(&compact);
    if !compact.is_ascii() {
        return Err(locale()
            .text("hex input must contain ASCII digits only")
            .into());
    }
    if compact.len() % 2 != 0 {
        return Err(locale()
            .text("hex input must contain an even number of digits")
            .into());
    }

    (0..compact.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&compact[index..index + 2], 16).map_err(|_| {
                locale().format(
                    "invalid hex byte '{}': position {index}",
                    &[compact[index..index + 2].to_string(), format!("{}", index)],
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

    #[test]
    fn formats_assessments_without_turning_readiness_into_support() {
        let assessment = EvidenceAssessment {
            vid: 0x1532,
            pid: 0x0099,
            sources: vec![
                razers_device_registry::upstream::EvidenceSource::OpenRazer,
                razers_device_registry::upstream::EvidenceSource::OpenRgb,
            ],
            name_agreement: razers_device_registry::upstream::EvidenceAgreement::Disagree,
            kind_agreement: razers_device_registry::upstream::EvidenceAgreement::Agree,
            matrix_agreement: razers_device_registry::upstream::EvidenceAgreement::Agree,
            protocol_agreement: razers_device_registry::upstream::EvidenceAgreement::NotComparable,
            readiness: EvidenceReadiness::Corroborated,
            openrazer_features: vec![UpstreamFeature::Dpi, UpstreamFeature::Lighting],
            irazer_support: None,
        };

        assert_eq!(
            format_evidence_assessment(&assessment),
            "1532:0099\treadiness=corroborated\tsources=OpenRazer,OpenRGB\tname=disagree\tkind=agree\tmatrix=agree\tprotocol=not-comparable\topenrazer-features=dpi,lighting\tirazer-support=absent"
        );
    }
}
