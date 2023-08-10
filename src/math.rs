use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use parry2d::na::{Isometry2, Vector2};
use vek::Vec2;

/// Position with a rotation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Iso {
    /// Position before being rotated.
    pub pos: Vec2<f32>,
    /// Rotation.
    pub rot: Rotation,
}

impl Iso {
    /// Construct from a position and a rotation.
    pub fn new<P, R>(pos: P, rot: R) -> Self
    where
        P: Into<Vec2<f32>>,
        R: Into<Rotation>,
    {
        let pos = pos.into();
        let rot = rot.into();

        Self { pos, rot }
    }

    /// Construct from a position with a rotation of zero.
    pub fn from_pos<P>(pos: P) -> Self
    where
        P: Into<Vec2<f32>>,
    {
        let pos = pos.into();
        let rot = Rotation::zero();

        Self { pos, rot }
    }

    /// Rotate a relative point and add the position.
    pub fn translate(&self, point: Vec2<f32>) -> Vec2<f32> {
        self.pos + self.rot.rotate(point)
    }
}

impl From<(Vec2<f32>, Rotation)> for Iso {
    fn from((pos, rot): (Vec2<f32>, Rotation)) -> Self {
        Self { pos, rot }
    }
}

impl From<Iso> for Isometry2<f32> {
    fn from(value: Iso) -> Self {
        Isometry2::new(
            Vector2::new(value.pos.x, value.pos.y),
            value.rot.to_radians(),
        )
    }
}

/// Rotation split into it's sine and cosine parts.
///
/// This allows something to rotate infinitely.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rotation {
    /// Cosine part of the rotation.
    cos: f32,
    /// Sine part of the rotation.
    sin: f32,
}

impl Rotation {
    /// With no rotation, points to the right.
    pub fn zero() -> Self {
        let (sin, cos) = (0.0, 0.0);

        Self { sin, cos }
    }

    /// Create from radians.
    pub fn from_radians(rotation: f32) -> Self {
        let (sin, cos) = rotation.sin_cos();

        Self { sin, cos }
    }

    /// Create from degrees.
    pub fn from_degrees(rotation: f32) -> Self {
        Self::from_radians(rotation.to_radians())
    }

    /// Create from a direction vector.
    ///
    /// Vector is assumed to be normalized.
    pub fn from_direction(dir: Vec2<f32>) -> Self {
        Self::from_radians(dir.y.atan2(dir.x))
    }

    /// Convert to radians.
    pub fn to_radians(self) -> f32 {
        self.sin.atan2(self.cos)
    }

    /// Convert to degrees.
    pub fn to_degrees(self) -> f32 {
        self.to_radians().to_degrees()
    }

    /// Rotate a point.
    pub fn rotate(&self, point: Vec2<f32>) -> Vec2<f32> {
        point.rotated_z(self.to_radians())
    }

    /// Sine.
    pub fn sin(&self) -> f32 {
        self.sin
    }

    /// Cosine.
    pub fn cos(&self) -> f32 {
        self.cos
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Self { cos: 1.0, sin: 0.0 }
    }
}

impl From<f32> for Rotation {
    fn from(value: f32) -> Self {
        Self::from_radians(value)
    }
}

impl AddAssign<f32> for Rotation {
    fn add_assign(&mut self, rhs: f32) {
        *self = *self + rhs;
    }
}

impl AddAssign<Self> for Rotation {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Add<f32> for Rotation {
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        if rhs.is_sign_positive() {
            self + Self::from_radians(rhs)
        } else {
            self - Self::from_radians(-rhs)
        }
    }
}

impl Add<Self> for Rotation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            cos: self.cos * rhs.cos - self.sin * rhs.sin,
            sin: self.sin * rhs.cos + self.cos * rhs.sin,
        }
    }
}

impl SubAssign<Self> for Rotation {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl SubAssign<f32> for Rotation {
    fn sub_assign(&mut self, rhs: f32) {
        *self = *self - rhs;
    }
}

impl Sub<Self> for Rotation {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl Sub<f32> for Rotation {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self::Output {
        if rhs.is_sign_positive() {
            self - Self::from_radians(rhs)
        } else {
            self + Self::from_radians(-rhs)
        }
    }
}

impl Neg for Rotation {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            cos: self.cos,
            sin: -self.sin,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Rotation;

    /// Test different operations on rotations.
    #[test]
    fn test_ops() {
        let mut a = Rotation::from_degrees(90.0);
        let b = Rotation::from_degrees(45.0);

        assert_eq!((-a).to_degrees().round() as i16, -90);
        assert_eq!((a + b).to_degrees().round() as i16, 135);
        assert_eq!((a - b).to_degrees().round() as i16, 45);

        assert_eq!((a + 45f32.to_radians()).to_degrees().round() as i16, 135);
        assert_eq!((a + 180f32.to_radians()).to_degrees().round() as i16, -90);
        assert_eq!((a - 180f32.to_radians()).to_degrees().round() as i16, -90);
        assert_eq!((a - 90f32.to_radians()).to_degrees().round() as i16, 0);
        assert_eq!(a - 1.0, a + -1.0);

        a -= 10f32.to_radians();
        assert_eq!(a.to_degrees().round() as i16, 80);
        a += 10f32.to_radians();
        assert_eq!(a.to_degrees().round() as i16, 90);
    }
}
