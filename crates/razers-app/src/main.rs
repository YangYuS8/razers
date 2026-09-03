// SPDX-License-Identifier: GPL-2.0-or-later
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui::{
    self, Align, Color32, CornerRadius, FontId, Frame, Layout, Margin, RichText, Stroke,
    ViewportBuilder,
};
use razers_app::discover_via_agent;
use razers_ipc::{DeviceList, DeviceSummary};

const ACCENT: Color32 = Color32::from_rgb(68, 214, 116);
const ACCENT_DARK: Color32 = Color32::from_rgb(19, 105, 52);

fn main() {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    match arguments.as_slice() {
        [] => {
            if let Err(error) = run_gui() {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
        [argument] if argument == "--agent-stdio" => {
            if let Err(error) =
                razers_agent::serve_stdio(std::io::stdin().lock(), std::io::stdout().lock())
            {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("error: unsupported RazeRS desktop argument");
            std::process::exit(2);
        }
    }
}

fn run_gui() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_app_id("io.github.yangyus8.razers")
            .with_title("RazeRS")
            .with_inner_size([920.0, 680.0])
            .with_min_inner_size([680.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "RazeRS",
        options,
        Box::new(|context| Ok(Box::new(RazersApp::new(context)))),
    )
}

struct RazersApp {
    discovery: Result<DeviceList, String>,
}

impl RazersApp {
    fn new(context: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*context.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(14.0, 8.0);
        context.egui_ctx.set_style(style);
        Self {
            discovery: discover_via_agent(),
        }
    }

    fn refresh(&mut self) {
        self.discovery = discover_via_agent();
    }
}

impl eframe::App for RazersApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header")
            .frame(
                Frame::new()
                    .inner_margin(Margin::symmetric(28, 18))
                    .fill(context.style().visuals.panel_fill),
            )
            .show(context, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("RazeRS")
                            .font(FontId::proportional(25.0))
                            .strong(),
                    );
                    ui.label(
                        RichText::new("READ-ONLY PREVIEW")
                            .small()
                            .strong()
                            .color(ACCENT),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let refresh = egui::Button::new(
                            RichText::new("Refresh devices")
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
            });

        egui::CentralPanel::default()
            .frame(Frame::central_panel(&context.style()).inner_margin(Margin::same(28)))
            .show(context, |ui| {
                ui.add_space(8.0);
                ui.label(RichText::new("Your devices").size(30.0).strong());
                ui.label(
                    RichText::new(
                        "A quiet, local place for your Razer hardware. No ads, accounts, or background tracking.",
                    )
                    .size(16.0)
                    .color(muted_text_color(ui)),
                );
                ui.add_space(18.0);

                match &self.discovery {
                    Ok(snapshot) if snapshot.devices.is_empty() => empty_state(ui),
                    Ok(snapshot) => {
                        ui.label(
                            RichText::new(format!(
                                "{} device {} across {} HID {}",
                                snapshot.devices.len(),
                                plural(snapshot.devices.len(), "identity", "identities"),
                                snapshot.interface_count,
                                plural(snapshot.interface_count, "interface", "interfaces")
                            ))
                            .color(muted_text_color(ui)),
                        );
                        ui.add_space(4.0);
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for device in &snapshot.devices {
                                device_card(ui, device);
                                ui.add_space(10.0);
                            }
                        });
                    }
                    Err(error) => error_state(ui, error),
                }

                ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                    ui.separator();
                    ui.label(
                        RichText::new(
                            "Private by design: device paths and serial-number values never enter this interface. Nothing is uploaded.",
                        )
                        .small()
                        .color(muted_text_color(ui)),
                    );
                });
            });
    }
}

fn empty_state(ui: &mut egui::Ui) {
    card_frame(ui).show(ui, |ui| {
        ui.set_min_height(180.0);
        ui.vertical_centered(|ui| {
            ui.add_space(28.0);
            ui.label(RichText::new("No Razer devices found").size(21.0).strong());
            ui.label(
                RichText::new("Connect a device, then choose Refresh devices.")
                    .color(muted_text_color(ui)),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new("RazeRS does not need an account or internet connection.")
                    .small()
                    .color(muted_text_color(ui)),
            );
        });
    });
}

fn error_state(ui: &mut egui::Ui, error: &str) {
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
            ui.label(RichText::new("Device discovery needs attention").strong());
            ui.label(error);
            ui.add_space(4.0);
            ui.label(
                RichText::new("No hardware was opened and no command was sent.")
                    .small()
                    .color(muted_text_color(ui)),
            );
        });
}

fn device_card(ui: &mut egui::Ui, device: &DeviceSummary) {
    card_frame(ui).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(&device.display_name).size(20.0).strong());
                ui.label(
                    RichText::new(format!(
                        "USB {}  ·  {} HID {}  ·  {} potential control {}",
                        device.usb_identity(),
                        device.interface_count,
                        plural(device.interface_count, "interface", "interfaces"),
                        device.vendor_interface_count,
                        plural(device.vendor_interface_count, "interface", "interfaces")
                    ))
                    .small()
                    .color(muted_text_color(ui)),
                );
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                status_badge(ui, &device.support_label);
            });
        });

        ui.add_space(8.0);
        ui.label(&device.support_detail);
        ui.label(
            RichText::new(&device.evidence_label)
                .small()
                .color(muted_text_color(ui)),
        );

        if !device.capabilities.is_empty() {
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Known capabilities").small().strong());
                for capability in &device.capabilities {
                    capability_badge(ui, capability);
                }
            });
        }

        ui.add_space(8.0);
        ui.add_enabled(device.control_available, egui::Button::new("Open controls"))
            .on_disabled_hover_text(
                "Controls stay unavailable until a typed, replay-tested driver is present.",
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

fn status_badge(ui: &mut egui::Ui, text: &str) {
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
            ui.label(RichText::new(text).small().strong().color(Color32::WHITE));
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

const fn plural<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pluralizes_user_facing_counts() {
        assert_eq!(plural(1, "device", "devices"), "device");
        assert_eq!(plural(0, "device", "devices"), "devices");
        assert_eq!(plural(2, "device", "devices"), "devices");
    }
}
