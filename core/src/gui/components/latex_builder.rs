use eframe::egui;
use std::fs::File;
use superpoweredcv::latex::LatexResume;
use superpoweredcv::generator::ScrapedProfile;
use crate::gui::types::InputSource;

/// Renders the LaTeX visual builder interface.
///
/// This component allows users to visually edit their resume content,
/// select templates, and configure formatting options.
///
/// # Arguments
///
/// * `ui` - The egui Ui context.
/// * `latex_resume` - The mutable state of the resume being built.
/// * `input_source` - The source of data (e.g., JSON file) to import from.
pub fn render_latex_builder(ui: &mut egui::Ui, latex_resume: &mut LatexResume, input_source: &InputSource) {
    ui.columns(2, |columns| {
        // Left Column: Editor
        columns[0].vertical(|ui| {
            render_editor_panel(ui, latex_resume, input_source);
        });

        // Right Column: Preview (Placeholder for now, or simplified view)
        columns[1].vertical(|ui| {
            render_preview_panel(ui, latex_resume);
        });
    });
}

fn render_editor_panel(ui: &mut egui::Ui, latex_resume: &mut LatexResume, input_source: &InputSource) {
    ui.heading(egui::RichText::new("Content Editor").color(egui::Color32::from_rgb(255, 69, 0)));
    
    ui.horizontal(|ui| {
        if ui.button("📥 Import from Input").clicked() {
            // Try to load from input source
            if let InputSource::JsonFile(Some(path)) = input_source {
                    if let Ok(file) = File::open(path) {
                        if let Ok(profile) = serde_json::from_reader::<_, ScrapedProfile>(file) {
                            latex_resume.import_from_profile(&profile);
                        }
                    }
            }
        }
        
        ui.label("Font:");
        egui::ComboBox::from_id_salt("font")
            .selected_text(if latex_resume.font.is_empty() { "Default" } else { &latex_resume.font })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut latex_resume.font, "".to_string(), "Default");
                ui.selectable_value(&mut latex_resume.font, "helvet".to_string(), "Helvetica");
                ui.selectable_value(&mut latex_resume.font, "mathpazo".to_string(), "Palatino");
                ui.selectable_value(&mut latex_resume.font, "avant".to_string(), "Avant Garde");
                ui.selectable_value(&mut latex_resume.font, "bookman".to_string(), "Bookman");
                ui.selectable_value(&mut latex_resume.font, "charter".to_string(), "Charter");
            });
    });

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.collapsing("Personal Info", |ui| {
            ui.horizontal(|ui| { ui.label("Name:"); ui.text_edit_singleline(&mut latex_resume.personal_info.name); });
            ui.horizontal(|ui| { ui.label("Email:"); ui.text_edit_singleline(&mut latex_resume.personal_info.email); });
            ui.horizontal(|ui| { ui.label("Phone:"); ui.text_edit_singleline(&mut latex_resume.personal_info.phone); });
            ui.horizontal(|ui| { ui.label("LinkedIn:"); ui.text_edit_singleline(&mut latex_resume.personal_info.linkedin); });
            ui.horizontal(|ui| { ui.label("GitHub:"); ui.text_edit_singleline(&mut latex_resume.personal_info.github); });
        });

        ui.separator();
        ui.heading(egui::RichText::new("Sections").color(egui::Color32::from_rgb(255, 69, 0)));
        ui.label(egui::RichText::new("Drag sections to reorder (Not implemented in this version)").small().italics());
        
        let mut section_to_remove = None;
        for (idx, section) in latex_resume.sections.iter_mut().enumerate() {
            ui.push_id(section.id.clone(), |ui| {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut section.title);
                        if ui.button("🗑").clicked() {
                            section_to_remove = Some(idx);
                        }
                    });
                    
                    for item in &mut section.items {
                        ui.push_id(item.id.clone(), |ui| {
                            let title = item.title.clone();
                            ui.collapsing(title, |ui| {
                                ui.text_edit_singleline(&mut item.title);
                                ui.text_edit_singleline(&mut item.subtitle);
                                ui.text_edit_singleline(&mut item.date);
                                for desc in &mut item.description {
                                    ui.text_edit_multiline(desc);
                                }
                            });
                        });
                    }
                });
            });
        }

        if let Some(idx) = section_to_remove {
            latex_resume.sections.remove(idx);
        }
    });
}

fn render_preview_panel(ui: &mut egui::Ui, latex_resume: &LatexResume) {
    ui.heading(egui::RichText::new("Live Preview").color(egui::Color32::from_rgb(255, 69, 0)));
    ui.separator();
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        // Simulate a paper view
        let paper_rect = ui.available_rect_before_wrap();
        let painter = ui.painter();
        
        // Background
        painter.rect_filled(paper_rect, 0.0, egui::Color32::WHITE);
        
        // Content (Simplified rendering of what the LaTeX might look like)
        ui.vertical(|ui| {
            ui.style_mut().visuals.override_text_color = Some(egui::Color32::BLACK);
            
            // Header
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new(&latex_resume.personal_info.name).size(24.0).strong());
                ui.label(format!("{} | {} | {}", 
                    latex_resume.personal_info.email, 
                    latex_resume.personal_info.phone,
                    latex_resume.personal_info.linkedin
                ));
            });
            
            ui.add_space(10.0);
            
            // Sections
            for section in &latex_resume.sections {
                ui.heading(egui::RichText::new(&section.title).size(16.0).strong().underline());
                for item in &section.items {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&item.title).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(&item.date).italics());
                        });
                    });
                    ui.label(egui::RichText::new(&item.subtitle).italics());
                    for desc in &item.description {
                        ui.label(format!("• {}", desc));
                    }
                    ui.add_space(5.0);
                }
                ui.add_space(10.0);
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use superpoweredcv::latex::{ResumeSection, SectionItem};

    #[test]
    fn test_latex_resume_structure() {
        let mut resume = LatexResume::default();
        resume.personal_info.name = "Test User".to_string();
        
        assert_eq!(resume.personal_info.name, "Test User");
        assert!(resume.sections.is_empty());
        
        resume.sections.push(ResumeSection {
            id: "1".to_string(),
            title: "Experience".to_string(),
            items: vec![SectionItem {
                id: "1-1".to_string(),
                title: "Job".to_string(),
                subtitle: "Company".to_string(),
                date: "2023".to_string(),
                description: vec!["Did stuff".to_string()],
            }],
        });
        
        assert_eq!(resume.sections.len(), 1);
        assert_eq!(resume.sections[0].items[0].title, "Job");
    }
}

