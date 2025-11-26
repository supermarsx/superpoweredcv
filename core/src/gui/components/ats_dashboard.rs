use eframe::egui;
use crate::config::AppConfig;
use crate::ats_simulation::{AtsSimulator, AtsSimulationResult};
use crate::pdf_utils::extract_text_from_pdf;
use std::path::PathBuf;

pub struct AtsDashboardState {
    pub selected_pdf: Option<PathBuf>,
    pub simulation_result: Option<AtsSimulationResult>,
    pub is_analyzing: bool,
    pub error_msg: Option<String>,
}

impl Default for AtsDashboardState {
    fn default() -> Self {
        Self {
            selected_pdf: None,
            simulation_result: None,
            is_analyzing: false,
            error_msg: None,
        }
    }
}

pub fn render_ats_dashboard(
    ui: &mut egui::Ui,
    state: &mut AtsDashboardState,
    config: &AppConfig,
) {
    ui.heading(egui::RichText::new("ATS / AI READ SIMULATION").size(20.0).strong().color(egui::Color32::from_rgb(255, 215, 0)));
    ui.add_space(10.0);
    ui.label("Simulate how an Applicant Tracking System (ATS) or AI parser sees your resume.");
    ui.add_space(10.0);

    ui.group(|ui| {
        ui.horizontal(|ui| {
            if ui.button("SELECT PDF TO ANALYZE").clicked() {
                if let Some(path) = rfd::FileDialog::new().add_filter("pdf", &["pdf"]).pick_file() {
                    state.selected_pdf = Some(path);
                    state.simulation_result = None;
                    state.error_msg = None;
                }
            }
            if let Some(path) = &state.selected_pdf {
                ui.label(path.file_name().unwrap_or_default().to_string_lossy());
            } else {
                ui.label("No file selected");
            }
        });

        if state.selected_pdf.is_some() {
            ui.add_space(10.0);
            if ui.button("RUN SIMULATION").clicked() {
                state.is_analyzing = true;
                state.error_msg = None;
                state.simulation_result = None;

                // Blocking call for now
                if let Some(path) = &state.selected_pdf {
                    match extract_text_from_pdf(path) {
                        Ok(text) => {
                            let simulator = AtsSimulator::new(config);
                            match simulator.simulate_parsing(&text) {
                                Ok(result) => {
                                    state.simulation_result = Some(result);
                                }
                                Err(e) => {
                                    state.error_msg = Some(format!("Simulation Error: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            state.error_msg = Some(format!("PDF Extraction Error: {}", e));
                        }
                    }
                }
                state.is_analyzing = false;
            }
        }
    });

    if let Some(err) = &state.error_msg {
        ui.label(egui::RichText::new(err).color(egui::Color32::RED));
    }

    if let Some(result) = &state.simulation_result {
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Analysis Results");
            ui.add_space(5.0);

            // Score
            ui.horizontal(|ui| {
                ui.label("Parsing Score:");
                let color = if result.parsing_score >= 90 {
                    egui::Color32::GREEN
                } else if result.parsing_score >= 70 {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::RED
                };
                ui.label(egui::RichText::new(format!("{} / 100", result.parsing_score)).size(18.0).strong().color(color));
            });

            ui.add_space(10.0);

            // Extracted Entities
            ui.group(|ui| {
                ui.label(egui::RichText::new("Extracted Entities").strong());
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.label(result.candidate_name.as_deref().unwrap_or("NOT FOUND"));
                });
                ui.horizontal(|ui| {
                    ui.label("Email:");
                    ui.label(result.email.as_deref().unwrap_or("NOT FOUND"));
                });
            });

            // Missing
            if !result.missing_entities.is_empty() {
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Missing / Unparsed").strong().color(egui::Color32::RED));
                    for missing in &result.missing_entities {
                        ui.label(format!("• {}", missing));
                    }
                });
            }

            // Skills
            ui.group(|ui| {
                ui.label(egui::RichText::new("Identified Skills").strong());
                ui.horizontal_wrapped(|ui| {
                    for skill in &result.skills_identified {
                        ui.label(egui::RichText::new(skill).code());
                    }
                });
            });

            // Timeline
            ui.group(|ui| {
                ui.label(egui::RichText::new("Reconstructed Timeline").strong());
                for exp in &result.experience_timeline {
                    ui.label(format!("• {} at {} ({})", exp.role, exp.company, exp.duration));
                }
            });
        });
    }
}
