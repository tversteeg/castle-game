use crate::unit::unit_type::UnitType;
use bevy::{
    core::{Name, Time},
    prelude::{Commands, Component, Query, Res},
};
use bevy_egui::egui::{Button, ProgressBar, Ui};
use bevy_inspector_egui::Inspectable;
use std::time::Duration;

/// The width of the button and the progress bar.
pub const WIDTH: f32 = 80.0;
pub const HEIGHT: f32 = 20.0;

/// A recruit button with a timer.
#[derive(Debug, Component, Inspectable)]
pub struct RecruitButton {
    /// What type of unit to recruit with this button.
    unit_type: UnitType,
    /// The time that already elapsed, will be reset when the unit is recruited.
    elapsed: Duration,
    /// When elapsed exceeds this the button can be pressed.
    time: Duration,
}

impl RecruitButton {
    /// Construct a new button.
    pub fn new(unit_type: UnitType, time: Duration) -> Self {
        Self {
            unit_type,
            elapsed: Duration::default(),
            time,
        }
    }

    /// Draw the button on the UI.
    pub fn draw(&mut self, ui: &mut Ui) -> Option<RecruitEvent> {
        let progress = self.progress();

        let mut event = None;

        ui.vertical(|ui| {
            if progress >= 1.0 {
                // The recruit button
                if ui
                    .add_sized([WIDTH, HEIGHT], Button::new(self.unit_type.to_string()))
                    .clicked()
                {
                    // Reset the time
                    self.elapsed = Duration::default();

                    // Throw the event for recruiting
                    event = Some(RecruitEvent(self.unit_type))
                }
            } else {
                // The progress bar
                ui.add_sized(
                    [WIDTH, HEIGHT],
                    ProgressBar::new(progress)
                        .text(self.unit_type.to_string())
                        .desired_width(WIDTH),
                );
            }
        });

        event
    }

    /// Get the progress as a fraction.
    fn progress(&self) -> f32 {
        let time_secs = self.time.as_secs_f32();
        let elapsed_secs = self.elapsed.as_secs_f32();

        1.0 - (time_secs - elapsed_secs) / time_secs
    }
}

/// The event for spawning a new unit.
#[derive(Debug, Clone)]
pub struct RecruitEvent(pub UnitType);

/// Count down the time.
pub fn system(mut query: Query<&mut RecruitButton>, time: Res<Time>) {
    for mut recruit_button in query.iter_mut() {
        if recruit_button.elapsed < recruit_button.time {
            // Subtract the time
            recruit_button.elapsed += time.delta();
        }
    }
}

/// Setup the recruit buttons.
pub fn setup(mut commands: Commands) {
    commands
        .spawn()
        .insert(RecruitButton::new(
            UnitType::Soldier,
            Duration::from_secs(2),
        ))
        .insert(Name::new("Soldier Recruit Button"));

    commands
        .spawn()
        .insert(RecruitButton::new(UnitType::Archer, Duration::from_secs(3)))
        .insert(Name::new("Archer Recruit Button"));
}
