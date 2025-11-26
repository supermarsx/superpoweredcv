use eframe::egui;
use rfd::FileDialog;
use std::path::PathBuf;
use superpoweredcv::generator::{self, ScrapedProfile};
use superpoweredcv::analysis::{ProfileConfig, InjectionPosition, Intensity, LowVisibilityPalette, OffpageOffset};
use std::fs::File;

pub fn run_gui() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 400.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "SUPERGUI",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            setup_custom_styles(&cc.egui_ctx);
            Ok(Box::new(MyApp::default()))
        }),
    )
}

struct MyApp {
    input_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    status_log: Vec<String>,
    
    // Injection
    injection_type: InjectionTypeGui,
    intensity: Intensity,
    position: InjectionPosition,
}

#[derive(PartialEq, Clone, Copy)]
enum InjectionTypeGui {
    None,
    VisibleMetaBlock,
    LowVisibilityBlock,
    OffpageLayer,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            input_path: None,
            output_path: None,
            status_log: vec!["> SYSTEM_READY".to_string()],
            injection_type: InjectionTypeGui::None,
            intensity: Intensity::Medium,
            position: InjectionPosition::Header,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.heading(egui::RichText::new("SUPERPOWERED_CV").size(32.0).strong().color(egui::Color32::from_rgb(0, 255, 65)));
                ui.add_space(10.0);
                ui.label(egui::RichText::new("TARGET: PDF_GENERATION_MODULE").monospace().color(egui::Color32::LIGHT_GRAY));
                ui.add_space(30.0);
            });

            // Input Section
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("INPUT_SOURCE:").strong().color(egui::Color32::WHITE));
                    if ui.button("SELECT_JSON").clicked() {
                        if let Some(path) = FileDialog::new().add_filter("json", &["json"]).pick_file() {
                            self.input_path = Some(path);
                            self.log("INPUT_SELECTED");
                        }
                    }
                    if let Some(path) = &self.input_path {
                        ui.label(egui::RichText::new(path.file_name().unwrap().to_string_lossy()).monospace().color(egui::Color32::YELLOW));
                    } else {
                        ui.label(egui::RichText::new("NO_FILE").monospace().color(egui::Color32::DARK_GRAY));
                    }
                });
            });

            ui.add_space(10.0);

            // Output Section
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("OUTPUT_DEST: ").strong().color(egui::Color32::WHITE));
                    if ui.button("SELECT_PATH").clicked() {
                        if let Some(path) = FileDialog::new().add_filter("pdf", &["pdf"]).save_file() {
                            self.output_path = Some(path);
                            self.log("OUTPUT_PATH_SET");
                        }
                    }
                    if let Some(path) = &self.output_path {
                        ui.label(egui::RichText::new(path.file_name().unwrap().to_string_lossy()).monospace().color(egui::Color32::YELLOW));
                    } else {
                        ui.label(egui::RichText::new("NO_PATH").monospace().color(egui::Color32::DARK_GRAY));
                    }
                });
            });

            ui.add_space(10.0);

            // Injection Section
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("INJECTION_MODULE:").strong().color(egui::Color32::WHITE));
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("TYPE:");
                        egui::ComboBox::from_id_salt("injection_type")
                            .selected_text(match self.injection_type {
                                InjectionTypeGui::None => "NONE",
                                InjectionTypeGui::VisibleMetaBlock => "VISIBLE_META",
                                InjectionTypeGui::LowVisibilityBlock => "LOW_VISIBILITY",
                                InjectionTypeGui::OffpageLayer => "OFF_PAGE",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.injection_type, InjectionTypeGui::None, "NONE");
                                ui.selectable_value(&mut self.injection_type, InjectionTypeGui::VisibleMetaBlock, "VISIBLE_META");
                                ui.selectable_value(&mut self.injection_type, InjectionTypeGui::LowVisibilityBlock, "LOW_VISIBILITY");
                                ui.selectable_value(&mut self.injection_type, InjectionTypeGui::OffpageLayer, "OFF_PAGE");
                            });
                    });

                    if self.injection_type != InjectionTypeGui::None {
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label("INTENSITY:");
                            egui::ComboBox::from_id_salt("intensity")
                                .selected_text(format!("{:?}", self.intensity).to_uppercase())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.intensity, Intensity::Soft, "SOFT");
                                    ui.selectable_value(&mut self.intensity, Intensity::Medium, "MEDIUM");
                                    ui.selectable_value(&mut self.intensity, Intensity::Aggressive, "AGGRESSIVE");
                                });
                        });

                        if self.injection_type == InjectionTypeGui::VisibleMetaBlock {
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("POSITION:");
                                let current_pos_text = match self.position {
                                    InjectionPosition::Header => "HEADER",
                                    InjectionPosition::Footer => "FOOTER",
                                    _ => "OTHER",
                                };
                                egui::ComboBox::from_id_salt("position")
                                    .selected_text(current_pos_text)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut self.position, InjectionPosition::Header, "HEADER");
                                        ui.selectable_value(&mut self.position, InjectionPosition::Footer, "FOOTER");
                                    });
                            });
                        }
                    }
                });
            });

            ui.add_space(30.0);

            // Action Button
            let generate_btn = egui::Button::new(egui::RichText::new("INITIATE_GENERATION").size(18.0).strong())
                .min_size(egui::vec2(ui.available_width(), 50.0))
                .fill(egui::Color32::from_rgb(0, 255, 65)); // Matrix Green

            if ui.add_enabled(self.input_path.is_some() && self.output_path.is_some(), generate_btn).clicked() {
                self.generate();
            }

            ui.add_space(20.0);

            // Console Log
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());
                egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    for log in &self.status_log {
                        ui.label(egui::RichText::new(log).monospace().size(12.0).color(egui::Color32::LIGHT_GRAY));
                    }
                });
            });
        });
    }
}

impl MyApp {
    fn log(&mut self, msg: &str) {
        self.status_log.push(format!("> {}", msg));
    }

    fn generate(&mut self) {
        self.log("STARTING_SEQUENCE...");
        
        let input = self.input_path.as_ref().unwrap();
        let output = self.output_path.as_ref().unwrap();

        let file = match File::open(input) {
            Ok(f) => f,
            Err(e) => {
                self.log(&format!("ERROR: FAILED_TO_OPEN_INPUT: {}", e));
                return;
            }
        };

        let profile: ScrapedProfile = match serde_json::from_reader(file) {
            Ok(p) => p,
            Err(e) => {
                self.log(&format!("ERROR: JSON_PARSE_FAIL: {}", e));
                return;
            }
        };

        let injection_config = match self.injection_type {
            InjectionTypeGui::None => None,
            InjectionTypeGui::VisibleMetaBlock => Some(ProfileConfig::VisibleMetaBlock {
                position: self.position.clone(),
                intensity: self.intensity.clone(),
            }),
            InjectionTypeGui::LowVisibilityBlock => Some(ProfileConfig::LowVisibilityBlock {
                font_size_min: 1,
                font_size_max: 1,
                color_profile: LowVisibilityPalette::Gray,
            }),
            InjectionTypeGui::OffpageLayer => Some(ProfileConfig::OffpageLayer {
                offset_strategy: OffpageOffset::BottomClip,
                length: None,
            }),
        };

        match generator::generate_pdf(&profile, output, injection_config.as_ref()) {
            Ok(_) => self.log("SUCCESS: PDF_GENERATED"),
            Err(e) => self.log(&format!("ERROR: GENERATION_FAIL: {}", e)),
        }
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // Get the font names used for Monospace
    if let Some(monospace_fonts) = fonts.families.get(&egui::FontFamily::Monospace) {
        // Set Proportional to use the same fonts
        fonts.families.insert(egui::FontFamily::Proportional, monospace_fonts.clone());
    }

    ctx.set_fonts(fonts);
}

fn setup_custom_styles(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    
    // Brutalist Colors
    visuals.window_fill = egui::Color32::from_rgb(10, 10, 10); // Almost black
    visuals.panel_fill = egui::Color32::from_rgb(10, 10, 10);
    
    // Sharp edges
    visuals.window_corner_radius = egui::CornerRadius::ZERO;
    visuals.menu_corner_radius = egui::CornerRadius::ZERO;
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::ZERO;
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::ZERO;
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::ZERO;
    visuals.widgets.active.corner_radius = egui::CornerRadius::ZERO;
    visuals.widgets.open.corner_radius = egui::CornerRadius::ZERO;

    // High Contrast Borders
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 50));
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 50));
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 65));

    // Button Colors
    visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
    visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(20, 20, 20);
    
    // Selection
    visuals.selection.bg_fill = egui::Color32::from_rgb(0, 255, 65);
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);

    ctx.set_visuals(visuals);
}
