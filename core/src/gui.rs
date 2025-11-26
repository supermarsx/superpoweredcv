pub mod types;
pub mod styles;
pub mod components;

use eframe::egui;
use std::path::PathBuf;
use std::fs::File;
use crate::generator::{self, ScrapedProfile};
use crate::attacks::{ProfileConfig, InjectionContent, LowVisibilityPalette, OffpageOffset, StructuralTarget, PaddingStyle, JobAdSource, JobAdPlacement};
use crate::attacks::templates::{GenerationType, default_templates};
use crate::config::AppConfig;
use crate::pdf::{PdfMutator, RealPdfMutator, PdfMutationRequest};
use crate::latex::LatexResume;

use self::types::{InputSource, LlmProvider, InjectionConfigGui, InjectionTypeGui, ProfileMask};
use self::styles::{setup_custom_fonts, setup_custom_styles, custom_window_frame};
use self::components::preview::render_preview;
use self::components::settings::render_settings;
use self::components::latex_builder::render_latex_builder;
use self::components::main_content::render_main_content;
use crate::gui::components::ai_assistant::{render_ai_assistant, AiAssistantState};
use crate::gui::components::ats_dashboard::{render_ats_dashboard, AtsDashboardState};

pub fn run_gui() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 800.0])
            .with_resizable(true)
            .with_decorations(false) // Custom window frame
            .with_transparent(true),
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
    input_source: InputSource,
    output_path: Option<PathBuf>,
    status_log: Vec<String>,
    
    // Injections
    injections: Vec<InjectionConfigGui>,
    
    // Config
    config: AppConfig,
    show_settings: bool,
    selected_provider: LlmProvider,

    // Latex Builder
    show_latex_builder: bool,
    latex_resume: LatexResume,

    // Log Window
    show_log_window: bool,

    // AI Assistant
    show_ai_assistant: bool,
    ai_assistant_state: AiAssistantState,

    // ATS Dashboard
    show_ats_dashboard: bool,
    ats_dashboard_state: AtsDashboardState,
    
    // Window States
    settings_pinned: bool,
    builder_pinned: bool,
    logs_pinned: bool,
    main_pinned: bool,
    preview_pinned: bool,
    ai_assistant_pinned: bool,
    ats_dashboard_pinned: bool,
    
    // Preview
    show_injection_preview: bool,

    // Profile Editor
    show_profile_editor: bool,
    profile_editor_pinned: bool,
    loaded_profile: Option<ScrapedProfile>,
    profile_mask: ProfileMask,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            input_source: InputSource::JsonFile(None),
            output_path: None,
            status_log: vec!["> SYSTEM_READY".to_string()],
            injections: vec![],
            config: AppConfig::load(),
            show_settings: false,
            selected_provider: LlmProvider::LMStudio,
            show_latex_builder: false,
            latex_resume: LatexResume::default(),
            show_log_window: false,
            show_ai_assistant: false,
            ai_assistant_state: AiAssistantState::default(),

            show_ats_dashboard: false,
            ats_dashboard_state: AtsDashboardState::default(),
            
            settings_pinned: false,
            builder_pinned: false,
            logs_pinned: false,
            main_pinned: false,
            preview_pinned: false,
            ai_assistant_pinned: false,
            ats_dashboard_pinned: false,
            
            show_injection_preview: false,

            show_profile_editor: false,
            profile_editor_pinned: false,
            loaded_profile: None,
            profile_mask: ProfileMask::default(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Main Window Custom Frame
        let mut pinned = self.main_pinned;
        custom_window_frame(ctx, "SUPERPOWERED_CV", |ui| {
            let mut action = None;
            let config_clone = self.config.clone();
            
            render_main_content(
                ui,
                &mut self.input_source,
                &mut self.output_path,
                &mut self.injections,
                &config_clone,
                &mut self.show_settings,
                &mut self.show_latex_builder,
                &mut self.show_log_window,
                &mut self.show_injection_preview,
                &mut self.show_ai_assistant,
                &mut self.show_ats_dashboard,
                |msg| self.status_log.push(format!("> {}", msg)),
                || { action = Some(()); },
                &mut self.loaded_profile,
                &mut self.profile_mask,
                |path| self.config.add_recent_file(&path),
            );
            
            if action.is_some() {
                self.generate();
            }

        }, &mut pinned);
        self.main_pinned = pinned;

        // Settings Window
        if self.show_settings {
            let mut pinned = self.settings_pinned;
            let mut builder = egui::ViewportBuilder::default()
                .with_title("CONFIGURATION_MATRIX")
                .with_inner_size([500.0, 600.0])
                .with_decorations(false)
                .with_transparent(true);
            
            if pinned {
                builder = builder.with_always_on_top();
            }

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("settings_viewport"),
                builder,
                |ctx, _class| {
                    custom_window_frame(ctx, "CONFIGURATION_MATRIX", |ui| {
                        render_settings(ui, &mut self.config, &mut self.selected_provider, |msg| self.status_log.push(format!("> {}", msg)));
                    }, &mut pinned);
                    
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_settings = false;
                    }
                }
            );
            self.settings_pinned = pinned;
        }

        // Latex Builder Window
        if self.show_latex_builder {
            let mut pinned = self.builder_pinned;
            let mut builder = egui::ViewportBuilder::default()
                .with_title("LATEX_VISUAL_BUILDER")
                .with_inner_size([1000.0, 800.0])
                .with_decorations(false)
                .with_transparent(true);
            
            if pinned {
                builder = builder.with_always_on_top();
            }

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("latex_builder_viewport"),
                builder,
                |ctx, _class| {
                    custom_window_frame(ctx, "LATEX_VISUAL_BUILDER", |ui| {
                        render_latex_builder(ui, &mut self.latex_resume, &self.input_source);
                    }, &mut pinned);
                    
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_latex_builder = false;
                    }
                }
            );
            self.builder_pinned = pinned;
        }

        // Log Window
        if self.show_log_window {
            let mut pinned = self.logs_pinned;
            let mut builder = egui::ViewportBuilder::default()
                .with_title("SYSTEM_LOGS")
                .with_inner_size([400.0, 500.0])
                .with_decorations(false)
                .with_transparent(true);
            
            if pinned {
                builder = builder.with_always_on_top();
            }

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("log_viewport"),
                builder,
                |ctx, _class| {
                    custom_window_frame(ctx, "SYSTEM_LOGS", |ui| {
                        ui.heading(egui::RichText::new("/// SYSTEM EVENT LOG ///").strong().color(egui::Color32::from_rgb(0, 255, 0)));
                        ui.separator();
                        egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                            for log in &self.status_log {
                                ui.label(egui::RichText::new(log).monospace().size(10.0));
                            }
                        });
                    }, &mut pinned);
                    
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_log_window = false;
                    }
                }
            );
            self.logs_pinned = pinned;
        }

        // AI Assistant Window
        if self.show_ai_assistant {
            let mut pinned = self.ai_assistant_pinned;
            let mut builder = egui::ViewportBuilder::default()
                .with_title("AI_ASSISTANT")
                .with_inner_size([500.0, 700.0])
                .with_decorations(false)
                .with_transparent(true);
            
            if pinned {
                builder = builder.with_always_on_top();
            }

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("ai_assistant_viewport"),
                builder,
                |ctx, _class| {
                    custom_window_frame(ctx, "AI_ASSISTANT", |ui| {
                        if let Some(profile) = &mut self.loaded_profile {
                            render_ai_assistant(ui, &mut self.ai_assistant_state, profile, &self.config, &mut |msg| self.status_log.push(format!("> {}", msg)));
                        } else {
                            ui.label("Please load a profile first.");
                        }
                    }, &mut pinned);
                    
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_ai_assistant = false;
                    }
                }
            );
            self.ai_assistant_pinned = pinned;
        }

        // ATS Dashboard Window
        if self.show_ats_dashboard {
            let mut pinned = self.ats_dashboard_pinned;
            let mut builder = egui::ViewportBuilder::default()
                .with_title("ATS_SIMULATION")
                .with_inner_size([600.0, 700.0])
                .with_decorations(false)
                .with_transparent(true);
            
            if pinned {
                builder = builder.with_always_on_top();
            }

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("ats_dashboard_viewport"),
                builder,
                |ctx, _class| {
                    custom_window_frame(ctx, "ATS_SIMULATION", |ui| {
                        render_ats_dashboard(ui, &mut self.ats_dashboard_state, &self.config);
                    }, &mut pinned);
                    
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_ats_dashboard = false;
                    }
                }
            );
            self.ats_dashboard_pinned = pinned;
        }

        // Preview Window
        if self.show_injection_preview {
            let mut pinned = self.preview_pinned;
            let mut builder = egui::ViewportBuilder::default()
                .with_title("INJECTION_PREVIEW")
                .with_inner_size([600.0, 800.0])
                .with_decorations(false)
                .with_transparent(true);
            
            if pinned {
                builder = builder.with_always_on_top();
            }

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("preview_viewport"),
                builder,
                |ctx, _class| {
                    custom_window_frame(ctx, "INJECTION_PREVIEW", |ui| {
                        render_preview(ui);
                    }, &mut pinned);
                    
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_injection_preview = false;
                    }
                }
            );
            self.preview_pinned = pinned;
        }
    }
}

impl MyApp {
    fn log(&mut self, msg: &str) {
        self.status_log.push(format!("> {}", msg));
    }

    fn generate(&mut self) {
        self.log("STARTING PIPELINE...");
        
        // 1. Determine Base PDF
        let base_pdf_path = match &self.input_source {
            InputSource::JsonFile(Some(path)) => {
                // Generate temp PDF from JSON
                let file = match File::open(path) {
                    Ok(f) => f,
                    Err(e) => { self.log(&format!("Error opening JSON: {}", e)); return; }
                };
                let mut profile: ScrapedProfile = match serde_json::from_reader(file) {
                    Ok(p) => p,
                    Err(e) => { self.log(&format!("Error parsing JSON: {}", e)); return; }
                };

                // Apply Profile Mask
                if let Some(loaded) = &self.loaded_profile {
                    // We use the loaded profile if available, as it might be the same one
                    // But wait, generate_pdf takes a profile.
                    // We should filter the profile based on the mask.
                    
                    let mut filtered_profile = loaded.clone(); // Assuming ScrapedProfile is Clone, which it is derived
                    
                    // Filter Experience
                    let mut new_exp = Vec::new();
                    for (i, exp) in filtered_profile.experience.iter().enumerate() {
                        if i < self.profile_mask.experience_enabled.len() && self.profile_mask.experience_enabled[i] {
                            new_exp.push(exp.clone()); // ScrapedExperience needs Clone
                        }
                    }
                    filtered_profile.experience = new_exp;

                    // Filter Education
                    let mut new_edu = Vec::new();
                    for (i, edu) in filtered_profile.education.iter().enumerate() {
                        if i < self.profile_mask.education_enabled.len() && self.profile_mask.education_enabled[i] {
                            new_edu.push(edu.clone()); // ScrapedEducation needs Clone
                        }
                    }
                    filtered_profile.education = new_edu;

                    // Filter Skills
                    let mut new_skills = Vec::new();
                    for (i, skill) in filtered_profile.skills.iter().enumerate() {
                        if i < self.profile_mask.skills_enabled.len() && self.profile_mask.skills_enabled[i] {
                            new_skills.push(skill.clone());
                        }
                    }
                    filtered_profile.skills = new_skills;
                    
                    profile = filtered_profile;
                }
                
                let temp_path = std::env::temp_dir().join("superpoweredcv_temp.pdf");
                if let Err(e) = generator::generate_pdf(&profile, &temp_path, None) {
                    self.log(&format!("Error generating base PDF: {}", e));
                    return;
                }
                temp_path
            }
            InputSource::PdfFile(Some(path)) => path.clone(),
            InputSource::LinkedinUrl(_) => {
                self.log("Error: URL input not implemented yet.");
                return;
            }
            _ => {
                self.log("Error: No input selected.");
                return;
            }
        };

        // 2. Build Profiles
        let mut profiles = Vec::new();
        for inj in &self.injections {
            let content = InjectionContent {
                phrases: inj.phrases.clone(),
                generation_type: inj.generation_type.clone(),
                job_description: if inj.generation_type == GenerationType::AdTargeted { Some(inj.job_description.clone()) } else { None },
            };

            let profile = match inj.injection_type {
                InjectionTypeGui::VisibleMetaBlock => ProfileConfig::VisibleMetaBlock {
                    position: inj.position.clone(),
                    intensity: inj.intensity.clone(),
                    content,
                },
                InjectionTypeGui::LowVisibilityBlock => ProfileConfig::LowVisibilityBlock {
                    font_size_min: 1,
                    font_size_max: 1,
                    color_profile: LowVisibilityPalette::Gray,
                    content,
                },
                InjectionTypeGui::OffpageLayer => ProfileConfig::OffpageLayer {
                    offset_strategy: OffpageOffset::BottomClip,
                    content,
                },
                InjectionTypeGui::UnderlayText => ProfileConfig::UnderlayText,
                InjectionTypeGui::StructuralFields => ProfileConfig::StructuralFields {
                    targets: vec![StructuralTarget::PdfTag], // Default for now
                },
                InjectionTypeGui::PaddingNoise => ProfileConfig::PaddingNoise {
                    padding_tokens_before: 100,
                    padding_tokens_after: 100,
                    padding_style: PaddingStyle::JobRelated,
                    content,
                },
                InjectionTypeGui::InlineJobAd => ProfileConfig::InlineJobAd {
                    job_ad_source: JobAdSource::Inline,
                    placement: JobAdPlacement::Back,
                    ad_excerpt_ratio: 1.0,
                    content,
                },
                InjectionTypeGui::TrackingPixel => ProfileConfig::TrackingPixel {
                    url: "https://canarytokens.org/pixel".to_string(), // Default placeholder
                },
                InjectionTypeGui::CodeInjection => ProfileConfig::CodeInjection {
                    payload: "alert('XSS')".to_string(), // Default placeholder
                },
            };
            profiles.push(profile);
        }

        // 3. Mutate
        let output = self.output_path.as_ref().unwrap();
        let mutator = RealPdfMutator::new(output.parent().unwrap());
        
        let request = PdfMutationRequest {
            base_pdf: base_pdf_path,
            profiles,
            template: default_templates().into_iter().find(|t| t.id == "default").unwrap_or_else(|| default_templates()[0].clone()),
            variant_id: Some(output.file_stem().unwrap().to_string_lossy().to_string()),
        };

        match mutator.mutate(request) {
            Ok(res) => {
                // Rename result to final output
                if let Err(e) = std::fs::rename(&res.mutated_pdf, output) {
                    self.log(&format!("Error moving file: {}", e));
                } else {
                    self.log("SUCCESS: PDF Generated & Injected.");
                }
            }
            Err(e) => self.log(&format!("Error mutating PDF: {}", e)),
        }
    }
}
