//! XPBD based physics engine.
//!
//! Based on: https://matthias-research.github.io/pages/publications/PBDBodies.pdf

pub mod collision;
pub mod constraint;
pub mod rigidbody;

use bvh_arena::{volumes::Aabb, Bvh};
use hecs::{Component, ComponentRef, Entity, Query, World};
use serde::Deserialize;
use vek::{Aabr, Vec2};

use crate::{
    math::Iso,
    physics::rigidbody::{Collider, Translation},
};

use self::{
    collision::{CollisionResponse, CollisionState},
    constraint::{penetration::PenetrationConstraint, Constraint},
    rigidbody::{
        Orientation, Position, RigidBodyHandle, RigidBodyQuery, RigidBodySystems, Velocity,
    },
};

/// Rigid body index type.
pub type RigidBodyKey = Entity;

/// Physics simulation state.
pub struct Physics {
    /// All entities.
    world: World,
    /// Rigidbody references and handles.
    rigidbodies: RigidBodySystems,
    /// Penetration constraints.
    penetration_constraints: Vec<PenetrationConstraint>,
    /// Cache of broad phase collisions.
    ///
    /// This is a performance optimization so the vector doesn't have to be allocated every step.
    broad_phase_collisions: Vec<(RigidBodyKey, RigidBodyKey)>,
    /// Narrow phase collision state cache.
    ///
    /// This is a performance optimization so the vector doesn't have to be allocated many times every step.
    narrow_phase_state: CollisionState<RigidBodyKey>,
}

impl Physics {
    /// Create the new state.
    pub fn new() -> Self {
        let world = World::default();
        let rigidbodies = RigidBodySystems::new();
        let broad_phase_collisions = Vec::new();
        let narrow_phase_state = CollisionState::new();
        let penetration_constraints = Vec::new();

        Self {
            world,
            rigidbodies,
            broad_phase_collisions,
            penetration_constraints,
            narrow_phase_state,
        }
    }

    /// Simulate a single step.
    pub fn step(&mut self, dt: f64) {
        puffin::profile_scope!("Physics step");

        let settings = &crate::settings().physics;
        let substeps = settings.substeps;

        // Deltatime for each sub-step
        let sub_dt = dt / substeps as f64;

        {
            puffin::profile_scope!("Remove dropped rigidbodies");

            // Destroy every rigidbody handle that has no references anymore
            self.rigidbodies.destroy_dropped(&mut self.world);
        }

        {
            puffin::profile_scope!("Reset constraints");

            // Reset every constraint for calculating the sub-steps since they are iterative
            self.reset_constraints();
        }

        {
            puffin::profile_scope!("Broad phase collision detection");

            // Do a broad phase collision check to get possible colliding pairs
            self.collision_broad_phase(dt);
        }

        for _ in 0..substeps {
            puffin::profile_scope!("Substep");

            // Integrate the rigidbodies, applying velocities and forces
            self.rigidbodies.integrate(&mut self.world, sub_dt);

            // Do a narrow-phase collision check and generate penetration constraints
            self.collision_narrow_phase();

            // Apply all constraints
            self.apply_constraints(sub_dt);

            // Solve the velocities
            self.rigidbodies.update_velocities(&mut self.world, sub_dt);

            // Solve collision velocities
            self.velocity_solve(sub_dt);

            // Apply translations to bodies
            self.rigidbodies.apply_translation(&mut self.world);
        }

        /*
        {
            puffin::profile_scope!("Mark sleeping");
            // Finalize velocity based on position offset
            self.rigidbodies
                .iter_mut()
                .for_each(|(_, rigidbody)| rigidbody.mark_sleeping(dt));
        }
        */
    }

    /// Remove every rigidbody.
    pub fn reset(&mut self) {
        self.world.clear();
    }

    /// Get the calculated collision pairs with collision information.
    pub fn colliding_rigid_bodies(&mut self) -> &[(RigidBodyKey, RigidBodyKey, CollisionResponse)] {
        &self.narrow_phase_state.collisions
    }

    /// Whether a rigidbody is still in the grid range.
    pub fn is_rigidbody_on_grid(&self, _rigidbody: &RigidBodyHandle) -> bool {
        // TODO
        true
    }

    /// Amount of rigidbodies currently registered.
    pub fn rigidbody_amount(&self) -> u32 {
        self.world.len()
    }

    /// Do a broad-phase collision pass to get possible pairs.
    ///
    /// Fills the list of broad-phase collisions.
    fn collision_broad_phase(&mut self, dt: f64) {
        puffin::profile_scope!("Broad phase");

        self.broad_phase_collisions.clear();

        // Construct a bounding volume hierarchy to find matching pairs
        let mut bvh: Bvh<RigidBodyKey, Aabb<2>> = Bvh::default();

        // Fill the hierachy
        for (entity, aabr) in self.predicted_aabrs(dt) {
            bvh.insert(
                entity,
                Aabb::from_min_max(
                    [aabr.min.x as f32, aabr.min.y as f32],
                    [aabr.max.x as f32, aabr.max.y as f32],
                ),
            );
        }

        puffin::profile_scope!("Transfer BVH pairs");

        // Put all pairs into a separate array
        bvh.for_each_overlaping_pair(|a, b| self.broad_phase_collisions.push((*a, *b)));
    }

    /// Do a narrow-phase collision pass to get all colliding objects.
    ///
    /// Fills the penetration constraint list and the list of collisions.
    fn collision_narrow_phase(&mut self) {
        self.narrow_phase_state.clear();

        // Narrow-phase with SAT
        for (a, b) in self.broad_phase_collisions.iter() {
            puffin::profile_scope!("Narrow collision");

            debug_assert_ne!(a, b);

            // Rigidbody A positions and shape
            let mut a_ref = self
                .world
                .query_one::<(&Collider, &Position, Option<&Translation>, &Orientation)>(*a)
                .expect("Rigidbody not found");
            let (a_shape, a_pos, a_trans, a_rot) = a_ref.get().unwrap();

            // Rigidbody B positions and shape
            let mut b_ref = self
                .world
                .query_one::<(&Collider, &Position, Option<&Translation>, &Orientation)>(*b)
                .expect("Rigidbody not found");
            let (b_shape, b_pos, b_trans, b_rot) = b_ref.get().unwrap();

            self.narrow_phase_state.detect(
                *a,
                &a_shape.0,
                Iso::new(
                    a_pos.0 + a_trans.map(|trans| trans.0).unwrap_or_default(),
                    a_rot.0,
                ),
                *b,
                &b_shape.0,
                Iso::new(
                    b_pos.0 + b_trans.map(|trans| trans.0).unwrap_or_default(),
                    b_rot.0,
                ),
            );
        }

        self.penetration_constraints.clear();

        {
            puffin::profile_scope!("Collision responses to penetration constraints");

            // Generate penetration constraint
            for (a, b, response) in self.narrow_phase_state.collisions.iter() {
                self.penetration_constraints
                    .push(PenetrationConstraint::new([*a, *b], response.clone()));
            }
        }
    }

    /// Debug information for all constraints.
    pub fn debug_info_constraints(&self) -> Vec<(Vec2<f64>, Vec2<f64>, CollisionResponse)> {
        puffin::profile_scope!("Debug constraint info");

        // Create an ECS view for the rigidbodies, this is good for random access and performance
        let mut rigidbody_query = self.world.query::<RigidBodyQuery>();
        let mut rigidbodies = rigidbody_query.view();

        self.penetration_constraints
            .iter()
            .map(|constraint| {
                let [a, b] = rigidbodies
                    .get_mut_n([constraint.a, constraint.b])
                    .map(|v| v.unwrap());

                (
                    a.local_to_world(constraint.a_attachment()),
                    b.local_to_world(constraint.b_attachment()),
                    constraint.response.clone(),
                )
            })
            .collect()
    }

    /// Debug information, all vertices from all rigid bodies.
    pub fn debug_info_vertices(&self) -> Vec<Vec<Vec2<f64>>> {
        puffin::profile_scope!("Debug vertices info");

        // Create an ECS view for the rigidbodies, this is good for random access and performance
        self.world
            .query::<(&Position, &Orientation, &Collider)>()
            .into_iter()
            .map(|(_id, (pos, rot, collider))| {
                let iso = Iso::new(pos.0, rot.0);

                collider.0.vertices(iso)
            })
            .collect()
    }

    /// Apply velocity corrections caused by friction and restitution.
    fn velocity_solve(&mut self, sub_dt: f64) {
        // Create an ECS view for the rigidbodies, this is good for random access and performance
        let mut rigidbody_query = self.world.query_mut::<RigidBodyQuery>();
        let mut rigidbodies = rigidbody_query.view();

        self.penetration_constraints
            .iter()
            .for_each(|constraint| constraint.solve_velocities(&mut rigidbodies, sub_dt));
    }

    fn reset_constraints(&self) {
        // TODO
    }

    fn apply_constraints(&mut self, sub_dt: f64) {
        // Create an ECS view for the rigidbodies, this is good for random access and performance
        let mut rigidbody_query = self.world.query_mut::<RigidBodyQuery>();
        let mut rigidbodies = rigidbody_query.view();

        self.penetration_constraints
            .iter_mut()
            .for_each(|constraint| constraint.solve(&mut rigidbodies, sub_dt));
    }

    /// Iterator over all predicted Axis-aligned bounding rectangles with a predicted future position added.
    ///
    /// WARNING: `dt` is not from the substep but from the full physics step.
    // PERF: make this an iterator
    fn predicted_aabrs(&self, dt: f64) -> Vec<(Entity, Aabr<f64>)> {
        /// How far away we predict the impulses to move us for checking the collision during the next full deltatime.
        const PREDICTED_POSITION_MULTIPLIER: f64 = 2.0;

        self.world
            .query::<(&Position, Option<&Velocity>, &Orientation, &Collider)>()
            .iter()
            .map(move |(id, (pos, vel, rot, shape))| {
                // First AABR at stationary position
                let mut aabr = shape.0.aabr(Iso::new(pos.0, rot.0));

                if let Some(vel) = vel {
                    // Expand to future AABR if not static
                    aabr.expand_to_contain(shape.0.aabr(Iso::new(
                        pos.0 + vel.0 * PREDICTED_POSITION_MULTIPLIER * dt,
                        rot.0,
                    )));
                }

                (id, aabr)
            })
            .collect()
    }

    /// Get a single entity value from a rigidbody.
    ///
    /// Throws an error when the entity doesn't exist or doesn't contain the component.
    #[inline(always)]
    fn rigidbody_value<'a, T>(&'a self, rigidbody: &RigidBodyHandle) -> T::Ref
    where
        T: ComponentRef<'a>,
    {
        self.world
            .get::<T>(rigidbody.entity())
            .expect("Entity does not exist or has value of type")
    }

    /// Get a single entity value from a rigidbody that might not be set.
    #[inline(always)]
    fn rigidbody_opt_value<'a, T>(&'a self, rigidbody: &RigidBodyHandle) -> Option<T::Ref>
    where
        T: ComponentRef<'a>,
    {
        self.world.get::<T>(rigidbody.entity()).ok()
    }

    /// Get a single mutable entity value from a rigidbody.
    #[inline(always)]
    fn rigidbody_set_value<T>(&mut self, rigidbody: &RigidBodyHandle, value: T)
    where
        T: Component,
    {
        self.world
            .insert_one(rigidbody.entity(), value)
            .expect("Entity does not exist or has value of type")
    }
}

/// Physics settings loaded from a file so it's easier to change them with hot-reloading.
#[derive(Deserialize)]
pub struct Settings {
    /// How many substeps are taken in a single step.
    pub substeps: u8,
    /// Gravity applied every frame.
    pub gravity: f64,
    /// Damping applied to the velocity every timestep.
    pub air_friction: f64,
    /// Dampling applied to the rotation every timestep.
    pub rotation_friction: f64,
}
