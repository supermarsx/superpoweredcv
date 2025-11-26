use eframe::egui;
use rfd::FileDialog;
use std::fs::File;
use superpoweredcv::latex::LatexResume;
use superpoweredcv::generator::ScrapedProfile;
use crate::gui::types::InputSource;

pub fn render_latex_builder(ui: &mut egui::Ui, latex_resume: &mut LatexResume, input_source: &InputSource) {
    ui.columns(2, |columns| {
        // Left Column: Editor
        columns[0].vertical(|ui| {
            ui.heading("Content Editor");
            
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
                    ui.text_edit_singleline(&mut latex_resume.personal_info.name).on_hover_text("Name");
                    ui.text_edit_singleline(&mut latex_resume.personal_info.email).on_hover_text("Email");
                    ui.text_edit_singleline(&mut latex_resume.personal_info.phone).on_hover_text("Phone");
                    ui.text_edit_singleline(&mut latex_resume.personal_info.linkedin).on_hover_text("LinkedIn");
                    ui.text_edit_singleline(&mut latex_resume.personal_info.github).on_hover_text("GitHub");
                });

                ui.separator();
                ui.heading("Sections");
                ui.label(egui::RichText::new("Drag sections to reorder (Not implemented in this version)").small().italics());
                
                let mut section_to_remove = None;
                let mut move_up = None;
                let mut move_down = None;

                for (idx, section) in latex_resume.sections.iter_mut().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!("#{}", idx+1));
                            if ui.button("⬆").clicked() { move_up = Some(idx); }
                            if ui.button("⬇").clicked() { move_down = Some(idx); }
                            ui.text_edit_singleline(&mut section.title);
                            if ui.button("X").clicked() {
                                section_to_remove = Some(idx);
                            }
                        });
                        
                        let mut item_to_remove = None;
                        for (i_idx, item) in section.items.iter_mut().enumerate() {
                            ui.separator();
                            ui.text_edit_singleline(&mut item.title).on_hover_text("Title");
                            ui.text_edit_singleline(&mut item.subtitle).on_hover_text("Subtitle");
                            ui.text_edit_singleline(&mut item.date).on_hover_text("Date");
                            
                            ui.label("Description Points:");
                            let mut desc_to_remove = None;
                            for (d_idx, desc) in item.description.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.text_edit_singleline(desc);
                                    if ui.button("-").clicked() {
                                        desc_to_remove = Some(d_idx);
                                    }
                                });
                            }
                            if let Some(d) = desc_to_remove {
                                item.description.remove(d);
                            }
                            if ui.button("+ Add Point").clicked() {
                                item.description.push(String::new());
                            }

                            if ui.button("Remove Item").clicked() {
                                item_to_remove = Some(i_idx);
                            }
                        }
                        if let Some(i) = item_to_remove {
                            section.items.remove(i);
                        }
                        if ui.button("+ Add Item").clicked() {
                            section.items.push(superpoweredcv::latex::SectionItem {
                                id: uuid::Uuid::new_v4().to_string(),
                                title: "New Item".to_string(),
                                subtitle: "Subtitle".to_string(),
                                date: "Date".to_string(),
                                description: vec![],
                            });
                        }
                    });
                }
                if let Some(s) = section_to_remove {
                    latex_resume.sections.remove(s);
                }
                if let Some(idx) = move_up {
                    if idx > 0 {
                        latex_resume.sections.swap(idx, idx - 1);
                    }
                }
                if let Some(idx) = move_down {
                    if idx < latex_resume.sections.len() - 1 {
                        latex_resume.sections.swap(idx, idx + 1);
                    }
                }
                
                if ui.button("+ Add Section").clicked() {
                    latex_resume.sections.push(superpoweredcv::latex::ResumeSection {
                        id: uuid::Uuid::new_v4().to_string(),
                        title: "New Section".to_string(),
                        items: vec![],
                    });
                }
            });
        });

        // Right Column: Preview
        columns[1].vertical(|ui| {
            ui.heading("LaTeX Preview");
            let latex_code = latex_resume.generate_latex();
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(egui::TextEdit::multiline(&mut latex_code.as_str()).code_editor().desired_width(f32::INFINITY));
            });
            
            ui.horizontal(|ui| {
                if ui.button("COPY TO CLIPBOARD").clicked() {
                    ui.ctx().copy_text(latex_code.clone());
                }
                if ui.button("EXPORT PDF").clicked() {
                    // Save to temp file and run pdflatex
                    if let Some(path) = FileDialog::new().set_file_name("resume.pdf").save_file() {
                        let tex_path = path.with_extension("tex");
                        if std::fs::write(&tex_path, &latex_code).is_ok() {
                            // Try to run pdflatex
                            match std::process::Command::new("pdflatex")
                                .arg("-output-directory")
                                .arg(path.parent().unwrap())
                                .arg(&tex_path)
                                .output() {
                                    Ok(output) => {
                                        if output.status.success() {
                                            // Log success
                                        } else {
                                            // Log failure
                                        }
                                    },
                                    Err(_) => {
                                        // Log failure
                                    }
                                }
                        }
                    }
                }
            });
        });
    });
}
