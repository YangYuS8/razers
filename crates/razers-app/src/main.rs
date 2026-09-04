// SPDX-License-Identifier: GPL-2.0-or-later
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui::{
    self, Align, Color32, CornerRadius, FontId, Frame, Layout, Margin, RichText, Stroke,
    ViewportBuilder,
};
use razers_app::{DiscoveryError, discover_via_agent};
use razers_i18n::{Language, Locale, language_args};
use razers_ipc::{DeviceList, DeviceSummary};

const ACCENT: Color32 = Color32::from_rgb(68, 214, 116);
const ACCENT_DARK: Color32 = Color32::from_rgb(19, 105, 52);

fn main() {
    let (language, arguments) =
        language_args(std::env::args().skip(1).collect()).unwrap_or_else(|error| {
            eprintln!(
                "{}: {}",
                Locale::system().text("error"),
                Locale::system().text(&error)
            );
            std::process::exit(2);
        });
    let locale = language.unwrap_or_default().resolve();
    match arguments.as_slice() {
        [] => {
            if let Err(error) = run_gui(language) {
                eprintln!("{}: {error}", locale.text("error"));
                std::process::exit(1);
            }
        }
        [argument] if argument == "--agent-stdio" => {
            if let Err(error) =
                razers_agent::serve_stdio(std::io::stdin().lock(), std::io::stdout().lock())
            {
                eprintln!("{}: {error}", locale.text("error"));
                std::process::exit(1);
            }
        }
        [argument] if argument == "--help" || argument == "-h" => {
            println!("{}", locale.text("app.help"));
        }
        _ => {
            eprintln!("{}", locale.text("Unsupported RazeRS desktop argument."));
            std::process::exit(2);
        }
    }
}

fn run_gui(language: Option<Language>) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_app_id("io.github.yangyus8.razers")
            .with_title("RazeRS")
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!(
                    "../../../assets/icons/razers.png"
                ))
                .expect("bundled application icon is valid"),
            )
            .with_inner_size([920.0, 680.0])
            .with_min_inner_size([680.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "RazeRS",
        options,
        Box::new(move |context| Ok(Box::new(RazersApp::new(context, language)))),
    )
}

struct RazersApp {
    discovery: Result<DeviceList, DiscoveryError>,
    language: Language,
}

impl RazersApp {
    fn new(context: &eframe::CreationContext<'_>, override_language: Option<Language>) -> Self {
        install_fonts(&context.egui_ctx);
        let language = preferred_language(context.storage, override_language);
        let mut style = (*context.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(14.0, 8.0);
        context.egui_ctx.set_style(style);
        Self {
            discovery: discover_via_agent(),
            language,
        }
    }

    fn refresh(&mut self) {
        self.discovery = discover_via_agent();
    }
}

impl eframe::App for RazersApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("language", self.language.code().into());
    }
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        let locale = self.language.resolve();
        egui::TopBottomPanel::top("header")
            .frame(
                Frame::new()
                    .inner_margin(Margin::symmetric(28, 18))
                    .fill(context.style().visuals.panel_fill),
            )
            .show(context, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new("RazeRS")
                            .font(FontId::proportional(25.0))
                            .strong(),
                    );
                    ui.label(
                        RichText::new(locale.text("READ-ONLY PREVIEW"))
                            .small()
                            .strong()
                            .color(ACCENT),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let refresh = egui::Button::new(
                            RichText::new(locale.text("Refresh devices"))
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(ACCENT_DARK)
                        .stroke(Stroke::new(1.0_f32, ACCENT));
                        if ui.add(refresh).clicked() {
                            self.refresh();
                        }
                    });
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(locale.text("Language"));
                    egui::ComboBox::from_id_salt("language")
                        .selected_text(match self.language {
                            Language::Auto => locale.text("System default"),
                            Language::English => "English",
                            Language::SimplifiedChinese => "简体中文",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.language,
                                Language::Auto,
                                locale.text("System default"),
                            );
                            ui.selectable_value(&mut self.language, Language::English, "English");
                            ui.selectable_value(
                                &mut self.language,
                                Language::SimplifiedChinese,
                                "简体中文",
                            );
                        });
                    ui.hyperlink_to(
                        locale.text("Documentation"),
                        match locale {
                            Locale::En => "https://yangyus8.top/razers/en/",
                            Locale::ZhCn => "https://yangyus8.top/razers/zh-CN/",
                        },
                    );
                });
            });

        egui::CentralPanel::default()
            .frame(Frame::central_panel(&context.style()).inner_margin(Margin::same(28)))
            .show(context, |ui| {
                ui.add_space(8.0);
                ui.label(RichText::new(locale.text("Your devices")).size(30.0).strong());
                ui.label(
                    RichText::new(
                        locale.text("A quiet, local place for your Razer hardware. No ads, accounts, or background tracking."),
                    )
                    .size(16.0)
                    .color(muted_text_color(ui)),
                );
                ui.add_space(18.0);

                match &self.discovery {
                    Ok(snapshot) if snapshot.devices.is_empty() => empty_state(ui, locale),
                    Ok(snapshot) => {
                        ui.label(
                            RichText::new(locale.format("Device identities: {0} · HID interfaces: {1}", &[snapshot.devices.len().to_string(), snapshot.interface_count.to_string()]))
                            .color(muted_text_color(ui)),
                        );
                        ui.add_space(4.0);
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for device in &snapshot.devices {
                                device_card(ui, device, locale);
                                ui.add_space(10.0);
                            }
                        });
                    }
                    Err(error) => error_state(ui, error, locale),
                }

                ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                    ui.separator();
                    ui.label(
                        RichText::new(
                            locale.text("Private by design: device paths and serial-number values never enter this interface. Nothing is uploaded."),
                        )
                        .small()
                        .color(muted_text_color(ui)),
                    );
                });
            });
    }
}

fn empty_state(ui: &mut egui::Ui, locale: Locale) {
    card_frame(ui).show(ui, |ui| {
        ui.set_min_height(180.0);
        ui.vertical_centered(|ui| {
            ui.add_space(28.0);
            ui.label(
                RichText::new(locale.text("No Razer devices found"))
                    .size(21.0)
                    .strong(),
            );
            ui.label(
                RichText::new(locale.text("Connect a device, then choose Refresh devices."))
                    .color(muted_text_color(ui)),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new(
                    locale.text("RazeRS does not need an account or internet connection."),
                )
                .small()
                .color(muted_text_color(ui)),
            );
        });
    });
}

fn error_state(ui: &mut egui::Ui, error: &DiscoveryError, locale: Locale) {
    let (fill, border) = if ui.visuals().dark_mode {
        (
            Color32::from_rgb(65, 29, 32),
            Color32::from_rgb(172, 70, 77),
        )
    } else {
        (
            Color32::from_rgb(255, 235, 236),
            Color32::from_rgb(190, 75, 83),
        )
    };
    Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0_f32, border))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(18))
        .show(ui, |ui| {
            ui.label(RichText::new(locale.text("Device discovery needs attention")).strong());
            ui.label(locale.text(error.message));
            if !error.detail.is_empty() {
                ui.collapsing(locale.text("Technical details"), |ui| {
                    ui.label(&error.detail);
                });
            }
            ui.add_space(4.0);
            ui.label(
                RichText::new(locale.text("No hardware was opened and no command was sent."))
                    .small()
                    .color(muted_text_color(ui)),
            );
        });
}

fn device_card(ui: &mut egui::Ui, device: &DeviceSummary, locale: Locale) {
    card_frame(ui).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(locale.text(&device.display_name))
                        .size(20.0)
                        .strong(),
                );
                ui.label(
                    RichText::new(locale.format(
                        "USB {0} · HID interfaces: {1} · Potential control interfaces: {2}",
                        &[
                            device.usb_identity(),
                            device.interface_count.to_string(),
                            device.vendor_interface_count.to_string(),
                        ],
                    ))
                    .small()
                    .color(muted_text_color(ui)),
                );
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                status_badge(ui, &device.support_label, locale);
            });
        });

        ui.add_space(8.0);
        ui.label(locale.text(&device.support_detail));
        ui.label(
            RichText::new(device.evidence_source_count.map_or_else(
                || locale.text(&device.evidence_label).to_owned(),
                |count| {
                    locale.format(
                        "Corroborated by {0} community sources",
                        &[count.to_string()],
                    )
                },
            ))
            .small()
            .color(muted_text_color(ui)),
        );

        if !device.capabilities.is_empty() {
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(locale.text("Known capabilities"))
                        .small()
                        .strong(),
                );
                for capability in &device.capabilities {
                    capability_badge(ui, locale.text(capability));
                }
            });
        }

        ui.add_space(8.0);
        ui.add_enabled(
            device.control_available,
            egui::Button::new(locale.text("Open controls")),
        )
        .on_disabled_hover_text(
            locale
                .text("Controls stay unavailable until a typed, replay-tested driver is present."),
        );
    });
}

fn card_frame(ui: &egui::Ui) -> Frame {
    Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .stroke(Stroke::new(
            1.0_f32,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(18))
}

fn status_badge(ui: &mut egui::Ui, text: &str, locale: Locale) {
    let fill = match text {
        "Verified" | "Detected" => ACCENT_DARK,
        "Experimental" => Color32::from_rgb(139, 91, 14),
        "Needs attention" => Color32::from_rgb(139, 45, 52),
        "Known device" => Color32::from_rgb(45, 86, 151),
        _ => Color32::from_rgb(81, 87, 96),
    };
    Frame::new()
        .fill(fill)
        .corner_radius(CornerRadius::same(9))
        .inner_margin(Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.label(
                RichText::new(locale.text(text))
                    .small()
                    .strong()
                    .color(Color32::WHITE),
            );
        });
}

fn capability_badge(ui: &mut egui::Ui, text: &str) {
    Frame::new()
        .fill(ui.visuals().widgets.inactive.bg_fill)
        .corner_radius(CornerRadius::same(7))
        .inner_margin(Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.label(RichText::new(text).small());
        });
}

fn muted_text_color(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(174, 178, 184)
    } else {
        Color32::from_rgb(78, 84, 92)
    }
}

fn install_fonts(context: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "noto-sans-sc".into(),
        egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/NotoSansSC-Regular.otf"
        ))
        .into(),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push("noto-sans-sc".into());
    }
    context.set_fonts(fonts);
}

fn preferred_language(
    storage: Option<&dyn eframe::Storage>,
    override_language: Option<Language>,
) -> Language {
    override_language
        .or_else(|| {
            storage
                .and_then(|storage| storage.get_string("language"))
                .and_then(|value| Language::parse(&value))
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::App;
    use std::collections::BTreeMap;

    #[derive(Default)]
    struct Storage(BTreeMap<String, String>);
    impl eframe::Storage for Storage {
        fn get_string(&self, key: &str) -> Option<String> {
            self.0.get(key).cloned()
        }
        fn set_string(&mut self, key: &str, value: String) {
            self.0.insert(key.into(), value);
        }
        fn flush(&mut self) {}
    }

    #[test]
    fn language_preference_survives_restart_and_cli_takes_priority() {
        let mut storage = Storage::default();
        let mut app = RazersApp {
            discovery: Ok(DeviceList {
                protocol_version: 1,
                devices: vec![],
                interface_count: 0,
            }),
            language: Language::SimplifiedChinese,
        };
        app.save(&mut storage);
        assert_eq!(
            preferred_language(Some(&storage), None),
            Language::SimplifiedChinese
        );
        assert_eq!(
            preferred_language(Some(&storage), Some(Language::English)),
            Language::English
        );
        assert_eq!(
            preferred_language(Some(&storage), Some(Language::Auto)),
            Language::Auto
        );
        storage
            .0
            .insert("language".into(), "obsolete-language".into());
        assert_eq!(preferred_language(Some(&storage), None), Language::Auto);
    }

    #[test]
    fn bundled_font_covers_both_catalogs_without_system_fonts() {
        let context = egui::Context::default();
        install_fonts(&context);
        let catalogs = [
            include_str!("../../razers-i18n/locales/en.json"),
            include_str!("../../razers-i18n/locales/zh-CN.json"),
        ];
        let _ = context.run(egui::RawInput::default(), |context| {
            context.fonts(|fonts| {
                for source in catalogs {
                    let messages: BTreeMap<String, String> = serde_json::from_str(source).unwrap();
                    for message in messages.values() {
                        for character in message.chars().filter(|c| !c.is_whitespace()) {
                            for font in [FontId::proportional(16.0), FontId::monospace(16.0)] {
                                assert!(
                                    fonts.has_glyph(&font, character),
                                    "missing glyph: {character}"
                                );
                            }
                        }
                    }
                }
            });
        });
    }
}
