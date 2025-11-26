use eframe::egui;

pub fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    if let Some(monospace_fonts) = fonts.families.get(&egui::FontFamily::Monospace) {
        fonts.families.insert(egui::FontFamily::Proportional, monospace_fonts.clone());
    }
    ctx.set_fonts(fonts);
}

pub fn setup_custom_styles(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    // Brutalist Palette
    let bg_color = egui::Color32::from_rgb(10, 10, 10);
    let fg_color = egui::Color32::from_rgb(255, 255, 255);
    let accent_color = egui::Color32::from_rgb(255, 69, 0); // Red-Orange (Fireish)
    let border_color = egui::Color32::from_rgb(255, 255, 255); // White borders for brutalist look

    visuals.window_fill = bg_color;
    visuals.panel_fill = bg_color;
    visuals.window_corner_radius = egui::CornerRadius::ZERO;
    visuals.window_stroke = egui::Stroke::new(2.0, border_color);
    
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(2.0, border_color);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, fg_color);
    
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(20, 20, 20);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(2.0, border_color);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, fg_color);
    
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(40, 40, 40);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(2.0, accent_color);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, fg_color);
    
    visuals.widgets.active.bg_fill = accent_color;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, fg_color);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
    
    visuals.selection.bg_fill = accent_color;
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
    
    ctx.set_visuals(visuals);
    
    // Spacing
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(15.0, 15.0);
    style.spacing.window_margin = egui::Margin::same(20);
    style.spacing.button_padding = egui::vec2(15.0, 10.0);
    ctx.set_style(style);
}

pub fn custom_window_frame(
    ctx: &egui::Context,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
    pinned: &mut bool,
) {
    use egui::*;
    let panel_frame = Frame {
        fill: ctx.style().visuals.window_fill(),
        corner_radius: 0.into(),
        stroke: ctx.style().visuals.window_stroke(),
        ..Default::default()
    };

    CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
        let app_rect = ui.max_rect();

        let title_bar_height = 32.0;
        let title_bar_rect = {
            let mut rect = app_rect;
            rect.max.y = rect.min.y + title_bar_height;
            rect
        };
        title_bar_ui(ui, title_bar_rect, title, pinned);

        let content_rect = {
            let mut rect = app_rect;
            rect.min.y = title_bar_rect.max.y;
            rect
        };
        
        let mut content_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(content_rect)
                .layout(*ui.layout())
        );
        add_contents(&mut content_ui);
    });
}

fn title_bar_ui(
    ui: &mut egui::Ui,
    title_bar_rect: egui::Rect,
    title: &str,
    pinned: &mut bool,
) {
    use egui::*;

    let painter = ui.painter();

    let title_bar_response = ui.interact(title_bar_rect, Id::new("title_bar"), Sense::click_and_drag());

    painter.rect_filled(
        title_bar_rect,
        CornerRadius {
            nw: 10,
            ne: 10,
            sw: 0,
            se: 0,
        },
        ui.visuals().widgets.inactive.bg_fill,
    );

    painter.text(
        title_bar_rect.center(),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(14.0),
        ui.visuals().text_color(),
    );

    painter.line_segment(
        [
            title_bar_rect.left_bottom() + vec2(1.0, 0.0),
            title_bar_rect.right_bottom() + vec2(-1.0, 0.0),
        ],
        ui.visuals().widgets.noninteractive.bg_stroke,
    );

    if title_bar_response.double_clicked() {
        ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(true));
    } else if title_bar_response.is_pointer_button_down_on() {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }

    ui.allocate_new_ui(UiBuilder::new().max_rect(title_bar_rect), |ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.visuals_mut().button_frame = false;
            ui.add_space(8.0);

            if ui.add(Button::new("❌").frame(false)).clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Close);
            }
            
            let (maximize_text, maximize_cmd) = if ui.input(|i| i.viewport().maximized.unwrap_or(false)) {
                ("🗗", ViewportCommand::Maximized(false))
            } else {
                ("🗖", ViewportCommand::Maximized(true))
            };

            if ui.add(Button::new(maximize_text).frame(false)).clicked() {
                ui.ctx().send_viewport_cmd(maximize_cmd);
            }

            if ui.add(Button::new("🗕").frame(false)).clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
            }
            
            let pin_text = if *pinned { "📌" } else { "📍" };
            if ui.add(Button::new(pin_text).frame(false)).clicked() {
                *pinned = !*pinned;
            }
        });
    });
}
