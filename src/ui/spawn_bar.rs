use super::recruit_button::RecruitEvent;

use crate::{constants::Constants, ui::recruit_button::RecruitButton};
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::{EventWriter, Query, Res, ResMut},
};
use bevy_egui::{
    egui::{Align2, Window},
    EguiContext,
};

/// Render the bar.
pub fn system(
    mut egui_context: ResMut<EguiContext>,
    diagnostics: Res<Diagnostics>,
    constants: Res<Constants>,
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
        .anchor(
            Align2::CENTER_TOP,
            [
                constants.ui.main_bar_offset.x,
                constants.ui.main_bar_offset.y,
            ],
        )
        .frame(super::theme::frame())
        .show(egui_context.ctx_mut(), |ui| {
            super::theme::apply_theme(ui);

            ui.horizontal_top(|ui| {
                // The buy section
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Recruit");
                        ui.horizontal(|ui| {
                            for mut recruit_button in query.iter_mut() {
                                if let Some(event) = recruit_button.draw(ui, &constants.ui) {
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
