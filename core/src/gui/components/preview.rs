use eframe::egui;

pub fn render_preview(ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("VISUAL_DIAGNOSTIC_MODE").strong());
    let (rect, _resp) = ui.allocate_at_least(ui.available_size(), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    
    // Draw Page Background
    painter.rect_filled(rect, 0.0, egui::Color32::WHITE);
    painter.rect_stroke(rect, 0.0, egui::Stroke::new(2.0, egui::Color32::BLACK), egui::StrokeKind::Inside);
    
    // Draw Dummy Text Lines
    for i in 0..30 {
        let y = rect.min.y + 40.0 + (i as f32 * 15.0);
        if y < rect.max.y - 40.0 {
            painter.line_segment(
                [egui::pos2(rect.min.x + 30.0, y), egui::pos2(rect.max.x - 30.0, y)],
                egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY)
            );
        }
    }

    // Draw Injections
    // Note: To draw injections, we need access to the injections list.
    // We'll need to pass that in.
}
