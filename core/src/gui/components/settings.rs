use eframe::egui;
use superpoweredcv::config::AppConfig;
use crate::gui::types::LlmProvider;

pub fn render_settings(ui: &mut egui::Ui, config: &mut AppConfig, selected_provider: &mut LlmProvider, mut log_fn: impl FnMut(&str)) {
    ui.heading("LLM Provider Settings");
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
                            config.llm.model = "gpt-4".to_string();
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
                            config.llm.model = "gemini-1.5-pro".to_string();
                        }
                        LlmProvider::Cohere => {
                            config.llm.api_base_url = "https://api.cohere.com/v1".to_string();
                            config.llm.model = "command-r-plus".to_string();
                        }
                        LlmProvider::DeepSeek => {
                            config.llm.api_base_url = "https://api.deepseek.com/v1".to_string();
                            config.llm.model = "deepseek-chat".to_string();
                        }
                        _ => {}
                    }
                }
            });
    });

    if matches!(selected_provider, LlmProvider::Ollama | LlmProvider::LMStudio | LlmProvider::LocalAI) {
        if ui.button("Auto-Detect Local Models").clicked() {
            log_fn("Checking localhost:11434 and localhost:1234...");
            log_fn("Auto-detection requires running service.");
        }
    }

    ui.separator();
    
    ui.label("API URL:");
    ui.text_edit_singleline(&mut config.llm.api_base_url);
    
    ui.label("Model Name:");
    ui.text_edit_singleline(&mut config.llm.model);
    
    ui.label("API Key:");
    let mut key = config.llm.api_key.clone().unwrap_or_default();
    ui.add(egui::TextEdit::singleline(&mut key).password(true));
    config.llm.api_key = if key.is_empty() { None } else { Some(key) };

    ui.separator();
    ui.heading("Prompt Templates");
    
    ui.label("Control Sequence Prompt:");
    ui.text_edit_multiline(&mut config.prompts.control_sequence_generation);
    
    ui.label("Pollution Skills Prompt:");
    ui.text_edit_multiline(&mut config.prompts.pollution_skills_generation);
    
    ui.label("Ad-Targeted Prompt:");
    ui.text_edit_multiline(&mut config.prompts.ad_targeted_pollution);

    ui.add_space(10.0);
    if ui.button("Save Configuration").clicked() {
        if let Err(e) = config.save() {
            log_fn(&format!("Config Save Error: {}", e));
        } else {
            log_fn("Configuration Saved.");
        }
    }
}
