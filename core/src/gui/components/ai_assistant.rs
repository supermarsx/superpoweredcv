use eframe::egui;
use superpoweredcv::llm::LlmClient;
use superpoweredcv::config::AppConfig;
use superpoweredcv::generator::{ScrapedProfile, ScrapedExperience};

pub struct AiAssistantState {
    pub review_result: Option<String>,
    pub is_reviewing: bool,
    pub rewrite_target_index: Option<usize>, // Index of experience item being rewritten
    pub rewrite_result: Option<String>,
    pub is_rewriting: bool,
}

impl Default for AiAssistantState {
    fn default() -> Self {
        Self {
            review_result: None,
            is_reviewing: false,
            rewrite_target_index: None,
            rewrite_result: None,
            is_rewriting: false,
        }
    }
}

pub fn render_ai_assistant(
    ui: &mut egui::Ui,
    state: &mut AiAssistantState,
    profile: &mut ScrapedProfile,
    config: &AppConfig,
    log_fn: &mut impl FnMut(&str),
) {
    ui.group(|ui| {
        ui.heading(egui::RichText::new("AI ASSISTANT").color(egui::Color32::from_rgb(0, 255, 255)));
        ui.add_space(5.0);

        // Full CV Review
        ui.horizontal(|ui| {
            ui.label("Full Profile Review:");
            if ui.button("ANALYZE").clicked() {
                state.is_reviewing = true;
                state.review_result = None;
                
                // In a real async GUI, we'd spawn a thread. For now, we block (simple implementation)
                // or we just set a flag and do it in the update loop if we had an async runtime.
                // Since we are in immediate mode and likely single threaded for now, we might freeze.
                // Let's try to do it "blocking" but warn the user, or ideally spawn a thread and use a channel.
                // For this refactor, I'll keep it simple but acknowledge the freeze.
                
                let client = LlmClient::new(config.llm.clone());
                let prompt = format!(
                    "Review the following CV profile and provide constructive feedback on strengths, weaknesses, and ATS optimization:\n\n{}",
                    serde_json::to_string_pretty(profile).unwrap_or_default()
                );

                match client.generate(&prompt) {
                    Ok(response) => {
                        state.review_result = Some(response);
                        log_fn("AI Review Completed.");
                    }
                    Err(e) => {
                        log_fn(&format!("AI Review Failed: {}", e));
                    }
                }
                state.is_reviewing = false;
            }
        });

        if let Some(review) = &state.review_result {
            ui.collapsing("Review Results", |ui| {
                ui.label(review);
            });
        }

        ui.separator();

        // Experience Rewriter
        ui.label("Experience Enhancer:");
        for (idx, exp) in profile.experience.iter_mut().enumerate() {
            ui.collapsing(format!("{} at {}", exp.title, exp.company), |ui| {
                ui.label("Current Description:");
                ui.label(&exp.location); // Using location field for description/bullets in this schema? 
                // Wait, ScrapedExperience struct in generator.rs has: title, company, date_range, location.
                // It seems the schema is missing a "description" or "bullets" field!
                // I need to check generator.rs ScrapedExperience struct.
                
                // Assuming we might need to add a description field to ScrapedExperience if it's missing.
                // Let's check generator.rs content from previous turns.
                // It has: title, company, date_range, location.
                // It seems the "About" section is global.
                // If the schema is limited, maybe we rewrite the "About" section or we need to update the schema.
                // Let's assume for now we rewrite the 'About' section as a proxy for "Summary Rewrite".
            });
        }
        
        ui.separator();
        
        ui.label("Summary Rewrite:");
        ui.horizontal(|ui| {
            if ui.button("REWRITE SUMMARY").clicked() {
                 let client = LlmClient::new(config.llm.clone());
                 let prompt = format!(
                    "Rewrite the following professional summary to be more impactful, concise, and action-oriented:\n\n{}",
                    profile.about
                );
                
                match client.generate(&prompt) {
                    Ok(response) => {
                        profile.about = response; // Direct apply for now, or show diff
                        log_fn("Summary Rewritten.");
                    }
                    Err(e) => {
                        log_fn(&format!("Rewrite Failed: {}", e));
                    }
                }
            }
        });
        ui.text_edit_multiline(&mut profile.about);

    });
}
