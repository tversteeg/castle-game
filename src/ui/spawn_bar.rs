use super::recruit_button::RecruitEvent;
use crate::color::Palette;
use crate::ui::recruit_button::RecruitButton;
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::{EventWriter, Query, Res, ResMut},
};
use bevy_egui::{
    egui::{epaint::Shadow, Align2, Frame, Window},
    EguiContext,
};

/// Render the bar.
pub fn system(
    mut egui_context: ResMut<EguiContext>,
    diagnostics: Res<Diagnostics>,
    mut query: Query<&mut RecruitButton>,
    mut event_writer: EventWriter<RecruitEvent>,
) {
    Window::new("Spawn Bar")
        .resizable(false)
        // Change the size to the contents
        .auto_sized()
        .collapsible(false)
        .title_bar(false)
        // Always dock the window to the center of bottom of the screen
        .anchor(Align2::CENTER_BOTTOM, [0.0, -5.0])
        .frame(Frame {
            // Rounded edges
            corner_radius: 3.0,
            // No shadow
            shadow: Shadow {
                extrusion: 0.0,
                ..Default::default()
            },
            fill: Palette::C26.into(),
            margin: (5.0, 5.0).into(),
            ..Default::default()
        })
        .show(egui_context.ctx_mut(), |ui| {
            // Set the style of the UI
            ui.visuals_mut().override_text_color = Some(Palette::C9.into());
            ui.visuals_mut().widgets.inactive.bg_fill = Palette::C24.into();
            ui.visuals_mut().widgets.hovered.bg_fill = Palette::C25.into();
            ui.visuals_mut().widgets.active.bg_fill = Palette::C23.into();

            ui.horizontal_top(|ui| {
                // The buy section
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Recruit");
                        ui.horizontal(|ui| {
                            for mut recruit_button in query.iter_mut() {
                                if let Some(event) = recruit_button.draw(ui) {
                                    // A unit should be recruited, throw the event
                                    event_writer.send(event);
                                }
                            }
                        });
                    });
                });

                // The FPS
                ui.group(|ui| {
                    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
                        if let Some(average) = fps.average() {
                            ui.small(format!("FPS: {:.2}", average));
                        }
                    }
                });
            });
        });
}
