use crate::color::Palette;
use bevy_egui::egui::{epaint::Shadow, style::Margin, Frame, Rounding, Ui};

/// Default settings for a frame.
pub fn frame() -> Frame {
    Frame {
        // Rounded edges
        rounding: Rounding::same(3.0),
        // No shadow
        shadow: Shadow {
            extrusion: 0.0,
            ..Default::default()
        },
        fill: Palette::C26.into(),
        margin: Margin::symmetric(5.0, 5.0),
        ..Default::default()
    }
}

/// Set the colors and other variables.
pub fn apply_theme(ui: &mut Ui) {
    // Set the style of the UI
    ui.visuals_mut().override_text_color = Some(Palette::C9.into());
    ui.visuals_mut().widgets.inactive.bg_fill = Palette::C24.into();
    ui.visuals_mut().widgets.hovered.bg_fill = Palette::C25.into();
    ui.visuals_mut().widgets.active.bg_fill = Palette::C23.into();
    ui.visuals_mut().widgets.noninteractive.bg_fill = Palette::C24.into();
    ui.visuals_mut().widgets.open.bg_fill = Palette::C24.into();

    ui.visuals_mut().widgets.inactive.fg_stroke.color = Palette::C2.into();
    ui.visuals_mut().widgets.hovered.fg_stroke.color = Palette::C2.into();
    ui.visuals_mut().widgets.active.fg_stroke.color = Palette::C2.into();
    ui.visuals_mut().widgets.noninteractive.fg_stroke.color = Palette::C2.into();
    ui.visuals_mut().widgets.open.fg_stroke.color = Palette::C24.into();

    ui.visuals_mut().widgets.inactive.bg_stroke.color = Palette::C24.into();
    ui.visuals_mut().widgets.hovered.bg_stroke.color = Palette::C24.into();
    ui.visuals_mut().widgets.active.bg_stroke.color = Palette::C25.into();
    ui.visuals_mut().widgets.noninteractive.bg_stroke.color = Palette::C24.into();
    ui.visuals_mut().widgets.open.bg_stroke.color = Palette::C24.into();

    ui.visuals_mut().selection.bg_fill = Palette::C27.into();
}
