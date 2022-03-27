use crate::inspector::Inspectable;

/// A random number between two numbers.
#[derive(Debug, Clone, Copy)]
pub struct RandomRange {
    /// Min value.
    pub min: f32,
    /// Max value.
    pub max: f32,
}

impl RandomRange {
    /// Generate a single number.
    pub fn gen(&self) -> f32 {
        self.multiply_fraction(fastrand::f32())
    }

    /// Put the fraction within the range.
    pub fn multiply_fraction(&self, fraction: f32) -> f32 {
        fraction * (self.max - self.min) + self.min
    }
}

#[cfg(feature = "inspector")]
pub mod inspector {
    use super::*;
    use bevy_inspector_egui::{
        egui::{
            plot::{Legend, Plot, Polygon, Value, Values},
            Color32, Grid, Slider, Ui, Vec2,
        },
        options::NumberAttributes,
        Context,
    };

    /// Draw the inspectable view for a bevy [`Mesh`].
    impl Inspectable for RandomRange {
        type Attributes = NumberAttributes<f32>;

        fn ui(&mut self, ui: &mut Ui, attributes: Self::Attributes, context: &mut Context) -> bool {
            ui.add(
                Slider::new(&mut self.min, attributes.min.unwrap_or(0.0)..=self.max)
                    .text("Min")
                    .suffix(attributes.suffix.clone()),
            );

            ui.add(
                Slider::new(&mut self.max, self.min..=attributes.max.unwrap_or(1000.0))
                    .text("Max")
                    .suffix(attributes.suffix),
            );

            true
        }
    }
}
