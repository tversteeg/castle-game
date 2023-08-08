use parry2d::{
    mass_properties::MassProperties,
    na::{DVector, Isometry2, Vector2},
    query::{ContactManifold, DefaultQueryDispatcher, PersistentQueryDispatcher},
    shape::{SharedShape, TypedShape},
};
use smallvec::SmallVec;
use vek::{Aabr, Extent2, Vec2};

use crate::math::Iso;

use super::CollisionResponse;

/// Distance at which the collisions will be detected before actually touching.
const PREDICTION_DISTANCE: f32 = 0.1;

/// Distance at which we count a collision as valid.
const MIN_PENETRATION_DISTANCE: f32 = 0.00001;

/// Different shapes.
#[derive(Clone)]
pub struct Shape(SharedShape);

impl Shape {
    /// Create a rectangle.
    pub fn rectangle(size: Extent2<f32>) -> Self {
        let shape = SharedShape::cuboid(size.w / 2.0, size.h / 2.0);

        Self(shape)
    }

    /// Create a horizontal heightmap.
    pub fn heightmap(heights: &[f32], spacing: f32) -> Self {
        puffin::profile_function!();

        let shape = SharedShape::heightfield(
            DVector::from_row_slice(heights),
            Vector2::new(spacing * (heights.len() - 1) as f32, 1.0),
        );

        Self(shape)
    }

    /// Axis aligned bounding box.
    pub fn aabr(&self, iso: Iso) -> Aabr<f32> {
        puffin::profile_function!();

        let aabb = self.0.compute_aabb(&iso.into());
        let min = Vec2::new(aabb.mins.x, aabb.mins.y);
        let max = Vec2::new(aabb.maxs.x, aabb.maxs.y);

        Aabr { min, max }
    }

    /// Collide with another shape.
    pub fn collides(&self, a_pos: Iso, b: &Self, b_pos: Iso) -> SmallVec<[CollisionResponse; 4]> {
        puffin::profile_function!();

        let a = self;

        // Check collision and return contact information
        let mut manifolds: Vec<ContactManifold<(), ()>> = Vec::new();

        {
            puffin::profile_scope!("Finding collision contacts");

            let a_pos_na: Isometry2<f32> = a_pos.into();
            let b_pos_na: Isometry2<f32> = b_pos.into();
            let ab_pos = a_pos_na.inv_mul(&b_pos_na);
            DefaultQueryDispatcher
                .contact_manifolds(
                    &ab_pos,
                    a.0.as_ref(),
                    b.0.as_ref(),
                    PREDICTION_DISTANCE,
                    &mut manifolds,
                    &mut None,
                )
                .expect("Collision failed");
        }

        manifolds
            .into_iter()
            // Ignore all empty manifolds
            .filter(|manifold| !manifold.points.is_empty())
            .map(|manifold| {
                puffin::profile_scope!("Mapping for borrowing");

                // Normal vector that always points to the same location globally
                let normal = a_pos
                    .rot
                    .rotate(Vec2::new(manifold.local_n1.x, manifold.local_n1.y));

                // PERF: remove allocation
                let contacts = manifold.contacts().to_vec();

                (contacts, normal)
            })
            .flat_map(|(contacts, normal)| {
                puffin::profile_scope!("Mapping all contacts in manifold");

                contacts.into_iter().filter_map(move |tracked_contact| {
                    puffin::profile_scope!("Manifold translation");

                    let local_contact_1 =
                        Vec2::new(tracked_contact.local_p1.x, tracked_contact.local_p1.y);
                    let local_contact_2 =
                        Vec2::new(tracked_contact.local_p2.x, tracked_contact.local_p2.y);
                    let penetration = -tracked_contact.dist;

                    // Ignore collisions where there's not enough penetration
                    if penetration >= MIN_PENETRATION_DISTANCE {
                        Some(CollisionResponse {
                            local_contact_1,
                            local_contact_2,
                            normal,
                            penetration,
                        })
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    /// Calculate different values based on the shape and density.
    pub fn mass_properties(&self, density: f32) -> MassProperties {
        self.0.mass_properties(density)
    }

    /// Get a list of vertices for the shape.
    pub fn vertices(&self, iso: Iso) -> Vec<Vec2<f32>> {
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
