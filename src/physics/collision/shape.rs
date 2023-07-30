use vek::{Aabr, Extent2, Vec2};

use crate::math::Rotation;

use super::sat::{CollisionResponse, NarrowCollision, Projection};

/// Orientable rectangle.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rectangle {
    /// Half size in each direction from the center point.
    pub half_size: Extent2<f32>,
}

impl Rectangle {
    /// Construct a new rectangle with a rotation of 0.
    pub fn new(size: Extent2<f32>) -> Self {
        let half_size = size / 2.0;

        Self { half_size }
    }

    /// Calculate the axis aligned bounding rectangle.
    pub fn aabr(&self, pos: Vec2<f32>, rot: f32) -> Aabr<f32> {
        let (sin, cos) = rot.sin_cos();
        let (abs_sin, abs_cos) = (sin.abs(), cos.abs());

        let width = self.half_size.w * abs_cos + self.half_size.h * abs_sin;
        let height = self.half_size.w * abs_sin + self.half_size.h * abs_cos;
        let size = Extent2::new(width, height);

        let min = pos - size;
        let max = pos + size;

        Aabr { min, max }
    }

    /// Calculate the 4 corner points.
    pub fn vertices(&self, pos: Vec2<f32>, rot: Rotation) -> [Vec2<f32>; 4] {
        let w_sin = self.half_size.w * rot.sin();
        let w_cos = self.half_size.w * rot.cos();
        let h_sin = self.half_size.h * rot.sin();
        let h_cos = self.half_size.h * rot.cos();

        [
            pos + Vec2::new(-w_cos + h_sin, -w_sin - h_cos),
            pos + Vec2::new(w_cos + h_sin, w_sin - h_cos),
            pos + Vec2::new(w_cos - h_sin, w_sin + h_cos),
            pos + Vec2::new(-w_cos - h_sin, -w_sin + h_cos),
        ]
    }

    /// Get the normal axes for each side.
    pub fn normal_axes(rot: Rotation) -> [Vec2<f32>; 2] {
        // Normalized direction vector
        let vec1 = rot.as_dir();
        // Perpendicular to the above
        let vec2 = Vec2::new(-vec1.y, vec1.x);

        [vec1, vec2]
    }

    /// Width of the shape.
    pub fn width(&self) -> f32 {
        self.half_size.w * 2.0
    }

    /// Width of the shape / 2.
    pub fn half_width(&self) -> f32 {
        self.half_size.w
    }

    /// Height of the shape.
    pub fn height(&self) -> f32 {
        self.half_size.h * 2.0
    }

    /// Height of the shape / 2.
    pub fn half_height(&self) -> f32 {
        self.half_size.h
    }
}

impl NarrowCollision for Rectangle {
    fn collide_rectangle(
        &self,
        pos: Vec2<f32>,
        rot: Rotation,
        other_rect: Rectangle,
        other_pos: Vec2<f32>,
        other_rot: Rotation,
    ) -> Option<CollisionResponse> {
        // Get the perp axes of both
        let (a_axes, b_axes) = (
            Rectangle::normal_axes(rot),
            Rectangle::normal_axes(other_rot),
        );

        // Get the corners for both
        let a_vertices = self.vertices(pos, rot);
        let b_vertices = other_rect.vertices(other_pos, other_rot);

        // Keep track of the axis with the smallest overlap
        let mut smallest_overlap = std::f32::INFINITY;
        // The value here doesn't matter because it will always be overwritten
        let mut smallest_axis = Vec2::default();
        // The closest vertex to the collision point
        let mut contact = Vec2::default();

        // Check if we have a collision
        for (index, axis) in a_axes.into_iter().chain(b_axes.into_iter()).enumerate() {
            let proj1 = Projection::project(a_vertices, axis);
            let proj2 = Projection::project(b_vertices, axis);

            if proj1.separated(proj2) {
                // There is an axis in between, no collision is possible
                return None;
            } else {
                // PERF: is it faster to do this else here or rerun the loop later when a collision is found?
                let overlap = proj1.overlap(proj2);
                if overlap < smallest_overlap {
                    smallest_overlap = overlap;

                    // Check to which shape the axis belongs
                    if index < a_axes.len() {
                        // Axis belongs to object A
                        if proj1.max <= proj2.max {
                            smallest_axis = -axis;
                            contact = proj2.min_vertex;
                        } else {
                            smallest_axis = axis;
                            contact = proj2.max_vertex;
                        }
                    } else {
                        // Axis belongs to object B
                        if proj1.max < proj2.max {
                            smallest_axis = -axis;
                            contact = proj1.max_vertex;
                        } else {
                            smallest_axis = axis;
                            contact = proj1.min_vertex;
                        }
                    }
                }
            }
        }

        // Collision found, all axes overlap

        let mtv = smallest_axis * smallest_overlap;

        Some(CollisionResponse { mtv, contact })
    }
}

#[cfg(test)]
mod tests {
    use vek::{Extent2, Vec2};

    use crate::{math::Rotation, physics::collision::sat::NarrowCollision};

    use super::Rectangle;

    /// Test different grid constructions.
    #[test]
    fn test_collision() {
        // Create two identical rectangles
        let a = Rectangle::new(Extent2::new(50.0, 100.0));
        let b = a;

        // Rotate one by 45 degrees
        let (rot_a, rot_b) = (Rotation::from_degrees(45.0), Rotation::from_degrees(0.0));
        let (pos_a, pos_b) = (Vec2::new(50.0, 50.0), Vec2::new(100.0, 120.0));

        // There shouldn't be a collision
        assert_eq!(a.collide_rectangle(pos_a, rot_a, b, pos_b, rot_b), None);

        // Changing the rotations should create a collision
        let (rot_a, rot_b) = (Rotation::from_degrees(80.0), Rotation::from_degrees(-45.0));
        assert!(a.collide_rectangle(pos_a, rot_a, b, pos_b, rot_b).is_some());

        // Now lets move the second one closer to hit the first one
        let pos_b = Vec2::new(80.0, 120.0);
        assert!(a.collide_rectangle(pos_a, rot_a, b, pos_b, rot_b).is_some());
    }
}
