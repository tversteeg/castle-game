use serde::Deserialize;

/// Either a number or a random range.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(untagged)]
pub enum RandomRangeF64 {
    /// Single value.
    Static(f64),
    /// Random range.
    Range { min: f64, max: f64 },
}

impl RandomRangeF64 {
    /// Calculate the value.
    pub fn value(&self) -> f64 {
        match self {
            RandomRangeF64::Static(val) => *val,
            RandomRangeF64::Range { min, max } => fastrand::f64() * (max - min) + min,
        }
    }
}

impl From<RandomRangeF64> for f64 {
    fn from(value: RandomRangeF64) -> Self {
        value.value()
    }
}
