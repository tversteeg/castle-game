use std::fmt::{Debug, Formatter, Result};

use parry2d_f64::{
    mass_properties::MassProperties,
    na::{DVector, Isometry2, Vector2},
    query::{DefaultQueryDispatcher, PersistentQueryDispatcher},
    shape::{SharedShape, TypedShape},
};

use vek::{Aabr, Extent2, Vec2};

use crate::math::Iso;

use super::{CollisionResponse, CollisionState};

/// Different shapes.
#[derive(Clone)]
pub struct Shape(SharedShape);

impl Shape {
    /// Create a rectangle.
    pub fn rectangle(size: Extent2<f64>) -> Self {
        let shape = SharedShape::cuboid(size.w / 2.0, size.h / 2.0);

        Self(shape)
    }

    /// Create a horizontal heightmap.
    pub fn heightmap(heights: &[f64], spacing: f64) -> Self {
        puffin::profile_function!();

        let shape = SharedShape::heightfield(
            DVector::from_row_slice(heights),
            Vector2::new(spacing * (heights.len() - 1) as f64, 1.0),
        );

        Self(shape)
    }

    /// Axis aligned bounding box.
    pub fn aabr(&self, iso: Iso) -> Aabr<f64> {
        puffin::profile_function!();

        let aabb = self.0.compute_aabb(&iso.into());
        let min = Vec2::new(aabb.mins.x, aabb.mins.y);
        let max = Vec2::new(aabb.maxs.x, aabb.maxs.y);

        Aabr { min, max }
    }

    /// Collide with another shape, pushing into a vector.
    pub fn push_collisions<K>(
        &self,
        a_pos: Iso,
        a_data: K,
        b: &Shape,
        b_pos: Iso,
        b_data: K,
        state: &mut CollisionState<K>,
    ) where
        K: Clone,
    {
        let a = self;

        let a_pos_na: Isometry2<f64> = a_pos.into();
        let b_pos_na: Isometry2<f64> = b_pos.into();
        let ab_pos = a_pos_na.inv_mul(&b_pos_na);

        {
            puffin::profile_scope!("Finding collision contacts");

            DefaultQueryDispatcher
                .contact_manifolds(
                    &ab_pos,
                    a.0.as_ref(),
                    b.0.as_ref(),
                    0.0,
                    &mut state.manifolds,
                    &mut None,
                )
                .expect("Collision failed");
        }

        puffin::profile_scope!("Mapping all contacts in manifold");
        for manifold in state.manifolds.iter() {
            if manifold.points.is_empty() {
                continue;
            }

            // Normal vector that always points to the same location globally
            let normal = a_pos
                .rot
                .rotate(Vec2::new(manifold.local_n1.x, manifold.local_n1.y));

            for tracked_contact in manifold.contacts().iter() {
                let local_contact_1 =
                    Vec2::new(tracked_contact.local_p1.x, tracked_contact.local_p1.y);
                let local_contact_2 =
                    Vec2::new(tracked_contact.local_p2.x, tracked_contact.local_p2.y);
                let penetration = -tracked_contact.dist;

                state.collisions.push((
                    a_data.clone(),
                    b_data.clone(),
                    CollisionResponse {
                        local_contact_1,
                        local_contact_2,
                        normal,
                        penetration,
                    },
                ));
            }
        }
    }

    /// Collide with another shape.
    ///
    /// This function is very inefficient, use [`Self::push_collisions`].
    pub fn collides(&self, a_pos: Iso, b: &Self, b_pos: Iso) -> Vec<CollisionResponse> {
        let mut collision_state = CollisionState::new();
        self.push_collisions(a_pos, 0, b, b_pos, 0, &mut collision_state);

        collision_state
            .collisions
            .into_iter()
            .map(|(_, _, response)| response)
            .collect()
    }

    /// Calculate different values based on the shape and density.
    pub fn mass_properties(&self, density: f64) -> MassProperties {
        self.0.mass_properties(density)
    }

    /// Get a list of vertices for the shape.
    pub fn vertices(&self, iso: Iso) -> Vec<Vec2<f64>> {
        match self.0.as_typed_shape() {
            TypedShape::Cuboid(rect) => rect.to_polyline(),
            TypedShape::HeightField(height) => height.to_polyline().0,
            _ => todo!(),
        }
        .into_iter()
        .map(|point| iso.translate(Vec2::new(point.x, point.y)))
        .collect()
    }
}

impl Default for Shape {
    fn default() -> Self {
        Self::rectangle(Extent2::new(1.0, 1.0))
    }
}

impl Debug for Shape {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        // TODO
        write!(f, "Shape")
    }
}
