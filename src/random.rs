use serde::Deserialize;

/// Either a number or a random range.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(untagged)]
pub enum RandomRangeF32 {
    /// Single value.
    Static(f32),
    /// Random range.
    Range { min: f32, max: f32 },
}

impl RandomRangeF32 {
    /// Calculate the value.
    pub fn value(&self) -> f32 {
        match self {
            RandomRangeF32::Static(val) => *val,
            RandomRangeF32::Range { min, max } => fastrand::f32() * (max - min) + min,
        }
    }
}

impl From<RandomRangeF32> for f32 {
    fn from(value: RandomRangeF32) -> Self {
        value.value()
    }
}
