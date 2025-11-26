use eframe::egui;
use rfd::FileDialog;
use std::path::PathBuf;
use crate::attacks::{InjectionPosition, Intensity};
use crate::attacks::templates::GenerationType;
use crate::llm::LlmClient;
use crate::config::AppConfig;
use crate::gui::types::{InputSource, InjectionConfigGui, InjectionTypeGui, ProfileMask};
use crate::generator::ScrapedProfile;

/// Renders the main content area of the application.
///
/// This function handles the primary workflow:
/// 1. Input selection (JSON, PDF, URL)
/// 2. Output path selection
/// 3. Injection module configuration
/// 4. Execution trigger
///
/// # Arguments
///
/// * `ui` - The egui Ui context.
/// * `input_source` - Mutable reference to the selected input source.
/// * `output_path` - Mutable reference to the output file path.
/// * `injections` - Mutable list of configured injection modules.
/// * `config` - Read-only reference to the application configuration.
/// * `show_settings` - Toggle for the settings window.
/// * `show_latex_builder` - Toggle for the LaTeX builder window.
/// * `show_log_window` - Toggle for the log window.
/// * `show_injection_preview` - Toggle for the injection preview window.
/// * `log_fn` - Callback for logging messages.
/// * `generate_fn` - Callback for triggering the generation process.
/// * `loaded_profile` - The currently loaded profile (if any).
/// * `profile_mask` - The mask for enabling/disabling profile sections.
/// * `update_history_fn` - Callback to update history.
pub fn render_main_content(
    ui: &mut egui::Ui,
    input_source: &mut InputSource,
    output_path: &mut Option<PathBuf>,
    injections: &mut Vec<InjectionConfigGui>,
    config: &AppConfig,
    show_settings: &mut bool,
    show_latex_builder: &mut bool,
    show_log_window: &mut bool,
    show_injection_preview: &mut bool,
    mut log_fn: impl FnMut(&str),
    mut generate_fn: impl FnMut(),
    loaded_profile: &mut Option<ScrapedProfile>,
    profile_mask: &mut ProfileMask,
    mut update_history_fn: impl FnMut(String),
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.vertical_centered(|ui| {
        ui.add_space(10.0);
        ui.heading(egui::RichText::new("SUPERPOWERED_CV").size(32.0).strong().color(egui::Color32::from_rgb(255, 69, 0)));
        ui.add_space(5.0);
        ui.label(egui::RichText::new("TARGET: PDF_GENERATION_MODULE").monospace().color(egui::Color32::WHITE));
        ui.add_space(20.0);
    });

    // Top Bar
    ui.horizontal(|ui| {
        if ui.button("⚙ SETTINGS").clicked() {
            *show_settings = true;
        }
        if ui.button("📝 LATEX BUILDER").clicked() {
            *show_latex_builder = true;
        }
        if ui.button("📋 LOGS").clicked() {
            *show_log_window = true;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new("v1.0.0-alpha").weak().small());
        });
    });
    ui.add_space(10.0);

    // Input Section
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new("INPUT_SOURCE").strong().color(egui::Color32::WHITE));
        ui.add_space(5.0);
        
        ui.horizontal(|ui| {
            ui.radio_value(input_source, InputSource::JsonFile(None), "JSON Profile");
            ui.radio_value(input_source, InputSource::PdfFile(None), "Existing PDF");
            ui.radio_value(input_source, InputSource::LinkedinUrl(String::new()), "LinkedIn URL");
        });

        ui.add_space(5.0);

        let mut log_msg = None;
        match input_source {
            InputSource::JsonFile(path) => {
                ui.horizontal(|ui| {
                    if ui.button("SELECT JSON").clicked() {
                        if let Some(p) = FileDialog::new().add_filter("json", &["json"]).pick_file() {
                            *path = Some(p.clone());
                            update_history_fn(p.to_string_lossy().to_string());
                            log_msg = Some("INPUT: JSON_SELECTED");
                            
                            // Load profile immediately
                            if let Ok(file) = std::fs::File::open(&p) {
                                if let Ok(profile) = serde_json::from_reader::<_, ScrapedProfile>(file) {
                                    // Initialize mask
                                    profile_mask.experience_enabled = vec![true; profile.experience.len()];
                                    profile_mask.education_enabled = vec![true; profile.education.len()];
                                    profile_mask.skills_enabled = vec![true; profile.skills.len()];
                                    *loaded_profile = Some(profile);
                                }
                            }
                        }
                    }
                    
                    // Recent Files Dropdown
                    egui::ComboBox::from_id_salt("recent_files")
                        .selected_text("Recent Files...")
                        .show_ui(ui, |ui| {
                            for recent in &config.history.recent_json_files {
                                if ui.selectable_label(false, recent).clicked() {
                                    let p = PathBuf::from(recent);
                                    *path = Some(p.clone());
                                    update_history_fn(recent.clone());
                                    log_msg = Some("INPUT: RECENT_JSON_SELECTED");
                                    
                                    // Load profile immediately
                                    if let Ok(file) = std::fs::File::open(&p) {
                                        if let Ok(profile) = serde_json::from_reader::<_, ScrapedProfile>(file) {
                                            // Initialize mask
                                            profile_mask.experience_enabled = vec![true; profile.experience.len()];
                                            profile_mask.education_enabled = vec![true; profile.education.len()];
                                            profile_mask.skills_enabled = vec![true; profile.skills.len()];
                                            *loaded_profile = Some(profile);
                                        }
                                    }
                                }
                            }
                        });

                    if let Some(p) = path {
                        ui.label(egui::RichText::new(p.file_name().unwrap().to_string_lossy()).color(egui::Color32::from_rgb(255, 69, 0)));
                    } else {
                        ui.label("No file selected");
                    }
                });

                // Profile Editor
                if let Some(profile) = loaded_profile {
                    ui.separator();
                    ui.collapsing("Profile Editor", |ui| {
                        ui.label(egui::RichText::new(format!("Loaded: {}", profile.name)).strong());
                        
                        ui.collapsing("Experience", |ui| {
                            for (i, exp) in profile.experience.iter().enumerate() {
                                if i < profile_mask.experience_enabled.len() {
                                    ui.checkbox(&mut profile_mask.experience_enabled[i], format!("{} at {}", exp.title, exp.company));
                                }
                            }
                        });
                        
                        ui.collapsing("Education", |ui| {
                            for (i, edu) in profile.education.iter().enumerate() {
                                if i < profile_mask.education_enabled.len() {
                                    ui.checkbox(&mut profile_mask.education_enabled[i], format!("{} - {}", edu.degree, edu.school));
                                }
                            }
                        });

                        ui.collapsing("Skills", |ui| {
                            for (i, skill) in profile.skills.iter().enumerate() {
                                if i < profile_mask.skills_enabled.len() {
                                    ui.checkbox(&mut profile_mask.skills_enabled[i], skill);
                                }
                            }
                        });
                    });
                }
            }
            InputSource::PdfFile(path) => {
                ui.horizontal(|ui| {
                    if ui.button("SELECT PDF").clicked() {
                        if let Some(p) = FileDialog::new().add_filter("pdf", &["pdf"]).pick_file() {
                            *path = Some(p);
                            log_msg = Some("INPUT: PDF_SELECTED");
                        }
                    }
                    if let Some(p) = path {
                        ui.label(egui::RichText::new(p.file_name().unwrap().to_string_lossy()).color(egui::Color32::from_rgb(255, 69, 0)));
                    } else {
                        ui.label("No file selected");
                    }
                });
            }
            InputSource::LinkedinUrl(url) => {
                ui.horizontal(|ui| {
                    ui.label("URL:");
                    ui.text_edit_singleline(url);
                });
                ui.label(egui::RichText::new("Note: URL scraping requires external browser extension.").small().italics());
            }
        }
        if let Some(msg) = log_msg {
            log_fn(msg);
        }
    });

    ui.add_space(10.0);

    // Output Section
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("OUTPUT_DEST: ").strong().color(egui::Color32::WHITE));
            if ui.button("SELECT PATH").clicked() {
                if let Some(path) = FileDialog::new().add_filter("pdf", &["pdf"]).save_file() {
                    *output_path = Some(path);
                    log_fn("OUTPUT_PATH_SET");
                }
            }
            if let Some(path) = output_path {
                ui.label(egui::RichText::new(path.file_name().unwrap().to_string_lossy()).monospace().color(egui::Color32::from_rgb(255, 69, 0)));
            } else {
                ui.label(egui::RichText::new("NO_PATH").monospace().color(egui::Color32::WHITE));
            }
        });
    });

    ui.add_space(10.0);

    // Injection Modules
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("INJECTION_MODULES").strong().color(egui::Color32::WHITE));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✚ ADD MODULE").clicked() {
                    injections.push(InjectionConfigGui::default());
                }
                if ui.button("👁 PREVIEW").clicked() {
                    *show_injection_preview = !*show_injection_preview;
                }
            });
        });
        
        ui.separator();

        egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
            let mut to_remove = None;
            let mut pending_error = None;
            for (idx, injection) in injections.iter_mut().enumerate() {
                ui.push_id(idx, |ui| {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("MODULE #{}", idx + 1)).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("REMOVE").clicked() {
                                    to_remove = Some(idx);
                                }
                            });
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Type:");
                            egui::ComboBox::from_id_salt("type")
                                .selected_text(format!("{:?}", injection.injection_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut injection.injection_type, InjectionTypeGui::VisibleMetaBlock, "Visible Meta");
                                    ui.selectable_value(&mut injection.injection_type, InjectionTypeGui::LowVisibilityBlock, "Low Visibility");
                                    ui.selectable_value(&mut injection.injection_type, InjectionTypeGui::OffpageLayer, "Off-Page Layer");
                                    ui.selectable_value(&mut injection.injection_type, InjectionTypeGui::UnderlayText, "Underlay Text");
                                    ui.selectable_value(&mut injection.injection_type, InjectionTypeGui::StructuralFields, "Structural Fields");
                                    ui.selectable_value(&mut injection.injection_type, InjectionTypeGui::PaddingNoise, "Padding Noise");
                                    ui.selectable_value(&mut injection.injection_type, InjectionTypeGui::InlineJobAd, "Inline Job Ad");
                                    ui.selectable_value(&mut injection.injection_type, InjectionTypeGui::TrackingPixel, "Tracking Pixel");
                                    ui.selectable_value(&mut injection.injection_type, InjectionTypeGui::CodeInjection, "Code Injection");
                                });
                            
                            ui.label("Intensity:");
                            egui::ComboBox::from_id_salt("intensity")
                                .selected_text(format!("{:?}", injection.intensity))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut injection.intensity, Intensity::Soft, "Soft");
                                    ui.selectable_value(&mut injection.intensity, Intensity::Medium, "Medium");
                                    ui.selectable_value(&mut injection.intensity, Intensity::Aggressive, "Aggressive");
                                    ui.selectable_value(&mut injection.intensity, Intensity::Custom, "Custom (No Template)");
                                });
                        });

                        if injection.injection_type == InjectionTypeGui::VisibleMetaBlock {
                            ui.horizontal(|ui| {
                                ui.label("Position:");
                                egui::ComboBox::from_id_salt("pos")
                                    .selected_text(format!("{:?}", injection.position))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut injection.position, InjectionPosition::Header, "Header");
                                        ui.selectable_value(&mut injection.position, InjectionPosition::Footer, "Footer");
                                    });
                            });
                        }

                        ui.collapsing("Content Configuration", |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Gen Mode:");
                                egui::ComboBox::from_id_salt("gen")
                                    .selected_text(format!("{:?}", injection.generation_type))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut injection.generation_type, GenerationType::Static, "Static");
                                        ui.selectable_value(&mut injection.generation_type, GenerationType::LlmControl, "LLM Control");
                                        ui.selectable_value(&mut injection.generation_type, GenerationType::Pollution, "Pollution");
                                        ui.selectable_value(&mut injection.generation_type, GenerationType::AdTargeted, "Ad Targeted");
                                    });
                            });

                            if injection.generation_type == GenerationType::AdTargeted {
                                ui.label("Job Description:");
                                ui.text_edit_multiline(&mut injection.job_description);
                            }

                            if injection.generation_type != GenerationType::Static {
                                if ui.button("GENERATE CONTENT (LLM)").clicked() {
                                    // Need to handle async or blocking call here. 
                                    // For now, we clone config and do it blocking (freezes UI briefly)
                                    let client = LlmClient::new(config.llm.clone());
                                    let prompt = match injection.generation_type {
                                        GenerationType::LlmControl => &config.prompts.control_sequence_generation,
                                        GenerationType::Pollution => &config.prompts.pollution_skills_generation,
                                        GenerationType::AdTargeted => &config.prompts.ad_targeted_pollution,
                                        _ => "",
                                    };
                                    let final_prompt = if injection.generation_type == GenerationType::AdTargeted {
                                        prompt.replace("{job_description}", &injection.job_description)
                                    } else {
                                        prompt.to_string()
                                    };
                                    
                                    match client.generate(&final_prompt) {
                                        Ok(c) => injection.phrases.push(c),
                                        Err(e) => pending_error = Some(format!("LLM Error: {}", e)),
                                    }
                                }
                            }

                            ui.label("Phrases:");
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(&mut injection.current_phrase);
                                if ui.button("Add").clicked() && !injection.current_phrase.is_empty() {
                                    injection.phrases.push(injection.current_phrase.clone());
                                    injection.current_phrase.clear();
                                }
                            });
                            for (_pi, p) in injection.phrases.iter().enumerate() {
                                ui.label(format!("• {}", p));
                            }
                        });
                    });
                });
            }
            
            if let Some(idx) = to_remove {
                injections.remove(idx);
            }
            if let Some(e) = pending_error {
                log_fn(&e);
            }
        });
    });

    ui.add_space(20.0);

    // Action Button
    ui.vertical_centered(|ui| {
        let btn = egui::Button::new(egui::RichText::new("⚡ INJECT & GENERATE ⚡").size(20.0).strong().color(egui::Color32::WHITE))
            .fill(egui::Color32::from_rgb(255, 69, 0))
            .min_size(egui::vec2(200.0, 50.0));
        
        if ui.add(btn).clicked() {
            generate_fn();
        }
    });
    });
}
