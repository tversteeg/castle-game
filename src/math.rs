use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use parry2d_f64::na::{Isometry2, Vector2};
use vek::Vec2;

/// Position with a rotation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Iso {
    /// Position before being rotated.
    pub pos: Vec2<f64>,
    /// Rotation.
    pub rot: Rotation,
}

impl Iso {
    /// Construct from a position and a rotation.
    pub fn new<P, R>(pos: P, rot: R) -> Self
    where
        P: Into<Vec2<f64>>,
        R: Into<Rotation>,
    {
        let pos = pos.into();
        let rot = rot.into();

        Self { pos, rot }
    }

    /// Construct from a position with a rotation of zero.
    pub fn from_pos<P>(pos: P) -> Self
    where
        P: Into<Vec2<f64>>,
    {
        let pos = pos.into();
        let rot = Rotation::zero();

        Self { pos, rot }
    }

    /// Rotate a relative point and add the position.
    #[inline]
    pub fn translate(&self, point: Vec2<f64>) -> Vec2<f64> {
        self.pos + self.rot.rotate(point)
    }
}

impl From<(Vec2<f64>, Rotation)> for Iso {
    fn from((pos, rot): (Vec2<f64>, Rotation)) -> Self {
        Self { pos, rot }
    }
}

impl From<Iso> for Isometry2<f64> {
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
    cos: f64,
    /// Sine part of the rotation.
    sin: f64,
}

impl Rotation {
    /// With no rotation, points to the right.
    #[inline]
    pub const fn zero() -> Self {
        let (sin, cos) = (0.0, 1.0);

        Self { sin, cos }
    }

    /// Create from radians.
    #[inline]
    pub fn from_radians(rotation: f64) -> Self {
        let (sin, cos) = rotation.sin_cos();

        Self { sin, cos }
    }

    /// Create from degrees.
    #[inline]
    pub fn from_degrees(rotation: f64) -> Self {
        Self::from_radians(rotation.to_radians())
    }

    /// Create from a direction vector.
    ///
    /// Vector is assumed to be normalized.
    #[inline]
    pub fn from_direction(dir: Vec2<f64>) -> Self {
        Self::from_radians(dir.y.atan2(dir.x))
    }

    /// Convert to radians.
    #[inline]
    pub fn to_radians(self) -> f64 {
        self.sin.atan2(self.cos)
    }

    /// Convert to degrees.
    #[inline]
    pub fn to_degrees(self) -> f64 {
        self.to_radians().to_degrees()
    }

    /// Rotate a point.
    #[inline]
    pub fn rotate(&self, point: Vec2<f64>) -> Vec2<f64> {
        point.rotated_z(self.to_radians())
    }

    /// Sine.
    #[inline]
    pub fn sin(&self) -> f64 {
        self.sin
    }

    /// Cosine.
    #[inline]
    pub fn cos(&self) -> f64 {
        self.cos
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Self { cos: 1.0, sin: 0.0 }
    }
}

impl From<f64> for Rotation {
    fn from(value: f64) -> Self {
        Self::from_radians(value)
    }
}

impl AddAssign<f64> for Rotation {
    fn add_assign(&mut self, rhs: f64) {
        *self = *self + rhs;
    }
}

impl AddAssign<Self> for Rotation {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Add<f64> for Rotation {
    type Output = Self;

    fn add(self, rhs: f64) -> Self::Output {
        if rhs.is_sign_positive() {
            self + Self::from_radians(rhs)
        } else {
            self - Self::from_radians(-rhs)
        }
    }
}

impl Add<Self> for Rotation {
    type Output = Self;

    #[inline]
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

impl SubAssign<f64> for Rotation {
    fn sub_assign(&mut self, rhs: f64) {
        *self = *self - rhs;
    }
}

impl Sub<Self> for Rotation {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl Sub<f64> for Rotation {
    type Output = Self;

    fn sub(self, rhs: f64) -> Self::Output {
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

        assert_eq!((a + 45f64.to_radians()).to_degrees().round() as i16, 135);
        assert_eq!((a + 180f64.to_radians()).to_degrees().round() as i16, -90);
        assert_eq!((a - 180f64.to_radians()).to_degrees().round() as i16, -90);
        assert_eq!((a - 90f64.to_radians()).to_degrees().round() as i16, 0);
        assert_eq!(a - 1.0, a + -1.0);

        a -= 10f64.to_radians();
        assert_eq!(a.to_degrees().round() as i16, 80);
        a += 10f64.to_radians();
        assert_eq!(a.to_degrees().round() as i16, 90);
    }
}
