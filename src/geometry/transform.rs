use bevy::{math::Quat, prelude::Transform};

/// Add transformation builder functions to a type.
pub trait TransformBuilder {
    /// Implementation for types for getting mutable access to the transform.
    fn transform_mut_ref(&'_ mut self) -> &'_ mut Transform;

    /// Set the X & Y position for the mesh.
    fn with_position(mut self, x: f32, y: f32) -> Self
    where
        Self: Sized,
    {
        self.transform_mut_ref().translation.x = x;
        self.transform_mut_ref().translation.y = y;

        self
    }

    /// Set the rotation in degrees.
    fn with_rotation(mut self, rotation: f32) -> Self
    where
        Self: Sized,
    {
        self.transform_mut_ref().rotation = Quat::from_rotation_z(rotation);

        self
    }

    /// Set the Z-index of the mesh.
    ///
    /// Z-indices range from `0.0..100.0`.
    fn with_z_index(mut self, z_index: f32) -> Self
    where
        Self: Sized,
    {
        self.transform_mut_ref().translation.z = z_index;

        self
    }
}
