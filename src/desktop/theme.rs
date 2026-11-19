use eframe::egui::{
    Color32, Context, FontId, Frame, Margin, Mesh, Painter, Rect, Stroke, TextStyle,
    ThemePreference, Visuals, pos2, vec2,
};

pub const CHROME: Color32 = Color32::from_rgb(0x1B, 0x1C, 0x1F);
pub const PANEL: Color32 = Color32::from_rgb(0x22, 0x24, 0x28);
pub const TOOLBAR: Color32 = Color32::from_rgb(0x28, 0x2A, 0x2E);
pub const CONTROL: Color32 = Color32::from_rgb(0x30, 0x33, 0x38);
pub const DIVIDER: Color32 = Color32::from_rgb(0x3D, 0x40, 0x46);
pub const CANVAS: Color32 = Color32::from_rgb(0x40, 0x40, 0x43);
pub const OBJECT: Color32 = Color32::from_rgb(0x30, 0x31, 0x36);
pub const TEXT: Color32 = Color32::from_rgb(0xF0, 0xF1, 0xF3);
pub const MUTED: Color32 = Color32::from_rgb(0xB5, 0xB9, 0xC1);
pub const WIRE: Color32 = Color32::from_rgb(0xD3, 0xD6, 0xDC);
pub const OUTLINE: Color32 = Color32::from_rgb(0xA2, 0xA6, 0xAE);
pub const FOCUS: Color32 = Color32::from_rgb(0x9A, 0xCA, 0xFF);
pub const HIERARCHY_SELECTION: Color32 = Color32::from_rgb(0x41, 0x71, 0x9C);
pub const BRAND: Color32 = Color32::from_rgb(0xF4, 0x9B, 0x44);
pub const FUNCTION: Color32 = Color32::from_rgb(0xA7, 0xB9, 0xFF);

pub fn bar(fill: Color32) -> Frame {
    Frame::new()
        .fill(fill)
        .inner_margin(Margin::symmetric(12, 0))
}

pub fn canvas_background(painter: &Painter, rect: Rect) {
    painter.rect_filled(rect, 0, CANVAS);
    let mut dots = Mesh::default();
    let mut y = rect.min.y + 10.0;
    while y < rect.max.y {
        let mut x = rect.min.x + 10.0;
        while x < rect.max.x {
            dots.add_colored_rect(
                Rect::from_center_size(pos2(x, y), vec2(1.5, 1.5)),
                Color32::from_rgb(0x54, 0x54, 0x57),
            );
            x += 20.0;
        }
        y += 20.0;
    }
    painter.add(dots);
}

pub fn apply(ctx: &Context) {
    let mut visuals = Visuals::dark();
    visuals.panel_fill = PANEL;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = CHROME;
    visuals.faint_bg_color = OBJECT;
    visuals.code_bg_color = OBJECT;
    visuals.override_text_color = Some(TEXT);
    visuals.weak_text_color = Some(MUTED);
    visuals.hyperlink_color = FOCUS;
    visuals.warn_fg_color = Color32::from_rgb(0xFF, 0xD1, 0x80);
    visuals.error_fg_color = Color32::from_rgb(0xFF, 0x9B, 0x9B);
    visuals.selection.bg_fill = Color32::from_rgb(0x29, 0x3B, 0x4F);
    visuals.selection.stroke = Stroke::new(1.5, FOCUS);
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
    ] {
        widget.bg_fill = CONTROL;
        widget.weak_bg_fill = CONTROL;
        widget.bg_stroke = Stroke::new(1.0, DIVIDER);
        widget.fg_stroke = Stroke::new(1.0, TEXT);
        widget.corner_radius = 6.into();
    }
    for widget in [
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.bg_fill = Color32::from_rgb(0x39, 0x3C, 0x42);
        widget.weak_bg_fill = widget.bg_fill;
        widget.bg_stroke = Stroke::new(1.5, FOCUS);
        widget.fg_stroke = Stroke::new(1.0, TEXT);
        widget.corner_radius = 6.into();
    }
    ctx.set_theme(ThemePreference::Dark);
    ctx.set_visuals(visuals);
    ctx.global_style_mut(|style| {
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(14.0));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::proportional(13.0));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(12.0));
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::proportional(20.0));
        style.spacing.interact_size.y = 26.0;
        style.spacing.item_spacing = vec2(6.0, 5.0);
        style.spacing.button_padding = vec2(8.0, 3.0);
    });
}
