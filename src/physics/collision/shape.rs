use arrayvec::ArrayVec;
use parry2d::{
    mass_properties::MassProperties,
    na::{DVector, Isometry2, Vector2},
    query::{ContactManifold, DefaultQueryDispatcher, PersistentQueryDispatcher},
    shape::{SharedShape, TypedShape},
};
use pixels::wgpu::PUSH_CONSTANT_ALIGNMENT;
use smallvec::SmallVec;
use vek::{Aabr, Extent2, Vec2};

use crate::math::{Iso, Rotation};

use super::CollisionResponse;

/// Distance at which the collisions will be detected before actually touching.
const PREDICTION_DISTANCE: f32 = 0.1;

/// Distance at which we count a collision as valid.
const MIN_PENETRATION_DISTANCE: f32 = 0.0001;

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
            Vector2::new(spacing * heights.len() as f32, 1.0),
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
            pos + Vec2::new(-w_cos - h_sin, -w_sin + h_cos),
            pos + Vec2::new(-w_cos + h_sin, -w_sin - h_cos),
            pos + Vec2::new(w_cos + h_sin, w_sin - h_cos),
            pos + Vec2::new(w_cos - h_sin, w_sin + h_cos),
        ]
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

    /// Create a parry2d cuboid from this rectangle.
    fn to_shared_shape(self) -> SharedShape {
        puffin::profile_function!();

        SharedShape::cuboid(self.half_width(), self.half_height())
    }

    /// Calculate inertia based on mass.
    pub fn inertia(&self, mass: f32) -> f32 {
        mass * (self.width().powi(2) + self.height().powi(2)) / 12.0
    }

    /// Handle the collision between this and another rectangle.
    fn collide_rectangle(
        &self,
        pos: Vec2<f32>,
        rot: Rotation,
        other_rect: &Rectangle,
        other_pos: Vec2<f32>,
        other_rot: Rotation,
    ) -> ArrayVec<CollisionResponse, 2> {
        puffin::profile_function!();

        let (a, b) = {
            puffin::profile_scope!("Creating parry shapes");

            // Don't use big distances for numerical stability
            (self.to_shared_shape(), other_rect.to_shared_shape())
        };

        debug_assert!(a.is_convex());
        debug_assert!(b.is_convex());

        let a_pos = Isometry2::new(Vector2::new(pos.x, pos.y), rot.to_radians());
        let b_pos = Isometry2::new(
            Vector2::new(other_pos.x, other_pos.y),
            other_rot.to_radians(),
        );

        // Check collision and return contact information
        let mut manifold = ContactManifold::<(), bool>::new();

        {
            puffin::profile_scope!("Finding collision contacts");

            let ab_pos = a_pos.inv_mul(&b_pos);
            DefaultQueryDispatcher
                .contact_manifold_convex_convex(
                    &ab_pos,
                    a.0.as_ref(),
                    b.0.as_ref(),
                    PREDICTION_DISTANCE,
                    &mut manifold,
                )
                .expect("Collision failed");
        }

        if manifold.points.is_empty() {
            // No collision found
            return ArrayVec::new();
        }

        // Normal vector that always points to the same location globally
        let normal = rot.rotate(Vec2::new(manifold.local_n1.x, manifold.local_n1.y));

        manifold
            .contacts()
            .iter()
            .filter_map(|tracked_contact| {
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
            .collect()
    }
}

/// List of vertical heights with a set spacing.
#[derive(Debug, Clone, Default)]
pub struct Heightmap {
    /// Y position of point spread out by spacing.
    pub heights: Vec<f32>,
    /// X pixels between each point.
    pub spacing: u8,
}

impl Heightmap {
    /// Construct from existing list.
    pub fn new(heights: Vec<f32>, spacing: u8) -> Self {
        Self { heights, spacing }
    }

    /// Bounding box, heightmaps can never be rotated.
    fn aabr(&self, pos: Vec2<f32>) -> Aabr<f32> {
        let min = pos;
        let max = pos + Vec2::new(self.heights.len() as f32 * self.spacing as f32, 256.0);

        Aabr { min, max }
    }

    /// List of vertices.
    fn vertices(&self, pos: Vec2<f32>) -> Vec<Vec2<f32>> {
        self.heights
            .iter()
            .enumerate()
            .map(|(index, height)| {
                pos + Vec2::new(index as f32 * self.spacing as f32, *height as f32)
            })
            .collect()
    }

    /// Create a parry2d heightmap from this rectangle.
    fn to_shared_shape(&self) -> SharedShape {
        puffin::profile_function!();

        SharedShape::heightfield(
            DVector::from_row_slice(&self.heights),
            Vector2::new(self.spacing as f32 * self.heights.len() as f32, 1.0),
        )
    }

    /// Handle the collision between this and a rectangle.
    fn collide_rectangle(
        &self,
        pos: Vec2<f32>,
        other_rect: &Rectangle,
        other_pos: Vec2<f32>,
        other_rot: Rotation,
    ) -> Vec<CollisionResponse> {
        puffin::profile_function!();

        let (a, b) = {
            puffin::profile_scope!("Creating parry shapes");

            // Don't use big distances for numerical stability
            (self.to_shared_shape(), other_rect.to_shared_shape())
        };

        let a_pos = Isometry2::new(Vector2::new(pos.x, pos.y), 0.0);
        let b_pos = Isometry2::new(
            Vector2::new(other_pos.x, other_pos.y),
            other_rot.to_radians(),
        );

        // Check collision and return contact information
        let mut manifolds: Vec<ContactManifold<(), ()>> = Vec::new();

        {
            puffin::profile_scope!("Finding collision contacts");

            let ab_pos = a_pos.inv_mul(&b_pos);
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
                // Normal vector that always points to the same location globally
                let normal = Vec2::new(manifold.local_n1.x, manifold.local_n1.y);

                // PERF: remove allocation
                let contacts = manifold.contacts().to_vec();

                (contacts, normal)
            })
            .flat_map(|(contacts, normal)| {
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
}
