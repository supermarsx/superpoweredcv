use eframe::egui;
use crate::config::AppConfig;
use crate::latex::manager::LatexManager;
use crate::gui::types::LlmProvider;

#[derive(PartialEq, Clone, Copy)]
enum SettingsTab {
    Llm,
    Prompts,
    Latex,
    General,
}

/// Renders the settings window with tabs for different configuration sections.
///
/// # Arguments
///
/// * `ui` - The egui Ui context.
/// * `config` - The mutable application configuration.
/// * `selected_provider` - The currently selected LLM provider.
/// * `log_fn` - A callback for logging status messages.
pub fn render_settings(ui: &mut egui::Ui, config: &mut AppConfig, selected_provider: &mut LlmProvider, mut log_fn: impl FnMut(&str)) {
    let mut current_tab = ui.data(|d| d.get_temp::<SettingsTab>(egui::Id::new("settings_tab"))).unwrap_or(SettingsTab::Llm);

    ui.horizontal(|ui| {
        ui.selectable_value(&mut current_tab, SettingsTab::Llm, "LLM Provider");
        ui.selectable_value(&mut current_tab, SettingsTab::Prompts, "Prompts");
        ui.selectable_value(&mut current_tab, SettingsTab::Latex, "LaTeX");
        ui.selectable_value(&mut current_tab, SettingsTab::General, "General");
    });
    ui.separator();

    ui.data_mut(|d| d.insert_temp(egui::Id::new("settings_tab"), current_tab));

    egui::ScrollArea::vertical().show(ui, |ui| {
        match current_tab {
            SettingsTab::Llm => render_llm_settings(ui, config, selected_provider),
            SettingsTab::Prompts => render_prompt_settings(ui, config),
            SettingsTab::Latex => render_latex_settings(ui, config, &mut log_fn),
            SettingsTab::General => render_general_settings(ui, config),
        }

        ui.add_space(20.0);
        if ui.button("Save Configuration").clicked() {
            if let Err(e) = config.save() {
                log_fn(&format!("Config Save Error: {}", e));
            } else {
                log_fn("Configuration Saved.");
            }
        }
    });
}



fn render_llm_settings(ui: &mut egui::Ui, config: &mut AppConfig, selected_provider: &mut LlmProvider) {
    ui.heading(egui::RichText::new("LLM Provider Settings").color(egui::Color32::from_rgb(255, 69, 0)));
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        ui.label("Provider:");
        egui::ComboBox::from_id_salt("provider")
            .selected_text(format!("{:?}", selected_provider))
            .show_ui(ui, |ui| {
                let mut changed = false;
                changed |= ui.selectable_value(selected_provider, LlmProvider::OpenAI, "OpenAI").clicked();
                changed |= ui.selectable_value(selected_provider, LlmProvider::Anthropic, "Anthropic").clicked();
                changed |= ui.selectable_value(selected_provider, LlmProvider::Mistral, "Mistral").clicked();
                changed |= ui.selectable_value(selected_provider, LlmProvider::Groq, "Groq").clicked();
                changed |= ui.selectable_value(selected_provider, LlmProvider::OpenRouter, "OpenRouter").clicked();
                changed |= ui.selectable_value(selected_provider, LlmProvider::LocalAI, "LocalAI").clicked();
                changed |= ui.selectable_value(selected_provider, LlmProvider::Ollama, "Ollama").clicked();
                changed |= ui.selectable_value(selected_provider, LlmProvider::LMStudio, "LM Studio").clicked();
                changed |= ui.selectable_value(selected_provider, LlmProvider::Gemini, "Gemini").clicked();
                changed |= ui.selectable_value(selected_provider, LlmProvider::Cohere, "Cohere").clicked();
                changed |= ui.selectable_value(selected_provider, LlmProvider::DeepSeek, "DeepSeek").clicked();
                changed |= ui.selectable_value(selected_provider, LlmProvider::Custom, "Custom").clicked();

                if changed {
                    match selected_provider {
                        LlmProvider::OpenAI => {
                            config.llm.api_base_url = "https://api.openai.com/v1".to_string();
                            config.llm.model = "gpt-4o".to_string();
                        }
                        LlmProvider::Anthropic => {
                            config.llm.api_base_url = "https://api.anthropic.com/v1".to_string();
                            config.llm.model = "claude-3-opus-20240229".to_string();
                        }
                        LlmProvider::Mistral => {
                            config.llm.api_base_url = "https://api.mistral.ai/v1".to_string();
                            config.llm.model = "mistral-large-latest".to_string();
                        }
                        LlmProvider::Groq => {
                            config.llm.api_base_url = "https://api.groq.com/openai/v1".to_string();
                            config.llm.model = "llama3-70b-8192".to_string();
                        }
                        LlmProvider::OpenRouter => {
                            config.llm.api_base_url = "https://openrouter.ai/api/v1".to_string();
                            config.llm.model = "openai/gpt-4o".to_string();
                        }
                        LlmProvider::LocalAI => {
                            config.llm.api_base_url = "http://localhost:8080/v1".to_string();
                            config.llm.model = "gpt-3.5-turbo".to_string();
                        }
                        LlmProvider::Ollama => {
                            config.llm.api_base_url = "http://localhost:11434/v1".to_string();
                            config.llm.model = "llama3".to_string();
                        }
                        LlmProvider::LMStudio => {
                            config.llm.api_base_url = "http://localhost:1234/v1".to_string();
                            config.llm.model = "local-model".to_string();
                        }
                        LlmProvider::Gemini => {
                            config.llm.api_base_url = "https://generativelanguage.googleapis.com/v1beta/openai".to_string();
                            config.llm.model = "gemini-1.5-pro-latest".to_string();
                        }
                        LlmProvider::Cohere => {
                            config.llm.api_base_url = "https://api.cohere.ai/v1".to_string();
                            config.llm.model = "command-r-plus".to_string();
                        }
                        LlmProvider::DeepSeek => {
                            config.llm.api_base_url = "https://api.deepseek.com/v1".to_string();
                            config.llm.model = "deepseek-chat".to_string();
                        }
                        LlmProvider::Custom => {
                            // Keep existing
                        }
                    }
                }
            });
    });

    ui.label("API Base URL:");
    ui.text_edit_singleline(&mut config.llm.api_base_url);

    ui.label("Model Name:");
    ui.text_edit_singleline(&mut config.llm.model);

    ui.label("API Key (Optional):");
    let mut api_key = config.llm.api_key.clone().unwrap_or_default();
    if ui.add(egui::TextEdit::singleline(&mut api_key).password(true)).changed() {
        config.llm.api_key = if api_key.is_empty() { None } else { Some(api_key) };
    }
}

fn render_prompt_settings(ui: &mut egui::Ui, config: &mut AppConfig) {
    ui.heading(egui::RichText::new("Prompt Templates").color(egui::Color32::from_rgb(255, 69, 0)));
    
    ui.label("Control Sequence Prompt:");
    ui.text_edit_multiline(&mut config.prompts.control_sequence_generation);
    
    ui.label("Pollution Skills Prompt:");
    ui.text_edit_multiline(&mut config.prompts.pollution_skills_generation);
    
    ui.label("Ad-Targeted Prompt:");
    ui.text_edit_multiline(&mut config.prompts.ad_targeted_pollution);
}

fn render_latex_settings(ui: &mut egui::Ui, config: &mut AppConfig, log_fn: &mut impl FnMut(&str)) {
    ui.heading(egui::RichText::new("LaTeX Environment").color(egui::Color32::from_rgb(255, 69, 0)));
    ui.add_space(10.0);

    ui.checkbox(&mut config.latex.auto_detect, "Auto-detect System LaTeX");
    
    ui.horizontal(|ui| {
        ui.label("Binary Path:");
        ui.text_edit_singleline(&mut config.latex.binary_path);
    });

    if config.latex.auto_detect {
        if ui.button("Run Auto-Detection").clicked() {
            if let Some(path) = LatexManager::auto_detect() {
                config.latex.binary_path = path;
                log_fn("LaTeX binary detected.");
            } else {
                log_fn("Could not detect LaTeX binary.");
            }
        }
    }

    ui.add_space(10.0);
    ui.label("Status:");
    if LatexManager::check_binary(&config.latex.binary_path) {
        ui.label(egui::RichText::new("● READY").color(egui::Color32::GREEN));
    } else {
        ui.label(egui::RichText::new("● NOT FOUND").color(egui::Color32::RED));
        ui.label("Please install a LaTeX distribution (TeX Live, MiKTeX, or Tectonic).");
    }
}

fn render_general_settings(ui: &mut egui::Ui, config: &mut AppConfig) {
    ui.heading(egui::RichText::new("General Settings").color(egui::Color32::from_rgb(255, 69, 0)));
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        ui.label("Max History Size:");
        ui.add(egui::DragValue::new(&mut config.history.max_history_size).range(1..=20));
    });

    ui.add_space(10.0);
    ui.label("Recent JSON Files:");
    
    let mut to_remove = None;
    for (idx, file) in config.history.recent_json_files.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("{}. {}", idx + 1, file));
            if ui.button("🗑").clicked() {
                to_remove = Some(idx);
            }
        });
    }

    if let Some(idx) = to_remove {
        config.history.recent_json_files.remove(idx);
    }

    if ui.button("Clear All History").clicked() {
        config.history.recent_json_files.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_latex_config() {
        let config = AppConfig::default();
        assert_eq!(config.latex.binary_path, "pdflatex");
        assert!(config.latex.auto_detect);
    }
}
