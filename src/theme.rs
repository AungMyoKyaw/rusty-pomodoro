//! Color themes. Reconstructed palettes, not copied assets.

use egui::Color32;

use crate::config::Theme as ThemeKind;

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub accent: Color32,
    pub background: Color32,
    pub panel: Color32,
    pub text: Color32,
    pub dim_text: Color32,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

pub fn palette(theme: ThemeKind) -> Palette {
    match theme {
        ThemeKind::Flamingo => Palette {
            accent: rgb(0xE8, 0x4A, 0x5F),
            background: rgb(0x1B, 0x14, 0x18),
            panel: rgb(0x2A, 0x1F, 0x24),
            text: rgb(0xF7, 0xEC, 0xEE),
            dim_text: rgb(0xB0, 0x9A, 0xA0),
        },
        ThemeKind::Mint => Palette {
            accent: rgb(0x3F, 0xC1, 0xA0),
            background: rgb(0x11, 0x1A, 0x18),
            panel: rgb(0x1C, 0x2A, 0x27),
            text: rgb(0xEC, 0xF7, 0xF3),
            dim_text: rgb(0x93, 0xAB, 0xA4),
        },
        ThemeKind::Mandarin => Palette {
            accent: rgb(0xF2, 0x8C, 0x28),
            background: rgb(0x1C, 0x16, 0x10),
            panel: rgb(0x2B, 0x22, 0x18),
            text: rgb(0xFA, 0xF1, 0xE6),
            dim_text: rgb(0xB4, 0xA2, 0x8C),
        },
        ThemeKind::Monochrome => Palette {
            accent: rgb(0xD8, 0xD8, 0xD8),
            background: rgb(0x14, 0x14, 0x14),
            panel: rgb(0x24, 0x24, 0x24),
            text: rgb(0xF0, 0xF0, 0xF0),
            dim_text: rgb(0x9A, 0x9A, 0x9A),
        },
        ThemeKind::Pastel => Palette {
            accent: rgb(0xA8, 0xC6, 0xE8),
            background: rgb(0x1A, 0x1C, 0x22),
            panel: rgb(0x28, 0x2B, 0x33),
            text: rgb(0xEE, 0xF1, 0xF6),
            dim_text: rgb(0x9D, 0xA4, 0xB2),
        },
    }
}

/// Push the palette into egui styles so widgets follow the theme.
pub fn apply(ctx: &egui::Context, palette: Palette) {
    let mut style = (*ctx.global_style()).clone();
    let mut visuals = style.visuals.clone();
    visuals.override_text_color = Some(palette.text);
    visuals.weak_text_color = Some(palette.dim_text);
    visuals.panel_fill = palette.background;
    visuals.window_fill = palette.panel;
    visuals.extreme_bg_color = palette.panel;
    visuals.faint_bg_color = palette.background;
    visuals.hyperlink_color = palette.accent;
    visuals.selection.bg_fill = palette.accent.gamma_multiply(0.35);
    visuals.selection.stroke = egui::Stroke::new(1.0, palette.accent);
    visuals.widgets.noninteractive.bg_fill = palette.panel;
    visuals.widgets.noninteractive.weak_bg_fill = palette.panel;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, palette.text);
    visuals.widgets.inactive.bg_fill = palette.panel;
    visuals.widgets.inactive.weak_bg_fill = palette.panel;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, palette.text);
    visuals.widgets.hovered.bg_fill = palette.accent.gamma_multiply(0.22);
    visuals.widgets.hovered.weak_bg_fill = palette.accent.gamma_multiply(0.22);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, palette.text);
    visuals.widgets.active.bg_fill = palette.accent.gamma_multiply(0.35);
    visuals.widgets.active.weak_bg_fill = palette.accent.gamma_multiply(0.35);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, palette.text);
    visuals.widgets.open.bg_fill = palette.accent.gamma_multiply(0.25);
    visuals.widgets.open.weak_bg_fill = palette.accent.gamma_multiply(0.25);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, palette.text);
    style.visuals = visuals;
    style
        .text_styles
        .insert(egui::TextStyle::Heading, egui::FontId::proportional(22.0));
    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::proportional(14.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, egui::FontId::proportional(14.0));
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(14.0, 9.0);
    style.spacing.window_margin = egui::Margin::same(12);
    style.visuals.window_corner_radius = egui::CornerRadius::same(10);
    style.visuals.menu_corner_radius = egui::CornerRadius::same(8);
    style.animation_time = 0.0;
    ctx.set_global_style(style);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_theme_kind_maps_to_a_palette() {
        for kind in crate::config::Theme::ALL {
            let palette = palette(kind);
            assert_ne!(palette.accent, palette.background);
        }
    }
}
