//! XPBD based physics engine.
//!
//! Based on: https://matthias-research.github.io/pages/publications/PBDBodies.pdf

pub mod collision;
pub mod constraint;
pub mod rigidbody;

use std::{
    ops::Deref,
    rc::{Rc, Weak},
};

use hecs::{Component, ComponentRef, Entity, Query, World};
use serde::Deserialize;
use slotmap::{DefaultKey, HopSlotMap, Key, KeyData};
use vek::{Aabr, Vec2};

use crate::math::{Iso, Rotation};

use self::{
    collision::{shape::Shape, spatial_grid::SpatialGrid, CollisionResponse, CollisionState},
    constraint::penetration::PenetrationConstraint,
    rigidbody::{
        AngularExternalForce, AngularVelocity, Orientation, Position, RigidBody, RigidBodyHandle,
        RigidBodySystems, Velocity,
    },
};

/// Rigid body index type.
pub type RigidBodyKey = Entity;
/// Distance constraint index type.
pub type DistanceConstraintKey = DefaultKey;

/// Physics simulation state.
pub struct Physics<
    const WIDTH: u16,
    const HEIGHT: u16,
    const STEP: u16,
    const BUCKET: usize,
    const SIZE: usize,
> {
    /// All entities.
    world: World,
    /// Rigidbody references and handles.
    rigidbodies: RigidBodySystems,
    /// Collision grid structure.
    collision_grid: SpatialGrid<RigidBodyKey, WIDTH, HEIGHT, STEP, BUCKET, SIZE>,
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

impl<
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const SIZE: usize,
    > Physics<WIDTH, HEIGHT, STEP, BUCKET, SIZE>
{
    /// Create the new state.
    pub fn new() -> Self {
        let world = World::default();
        let rigidbodies = RigidBodySystems::new();
        let collision_grid = SpatialGrid::new();
        let broad_phase_collisions = Vec::with_capacity(16);
        let narrow_phase_state = CollisionState::new();
        let penetration_constraints = Vec::new();

        Self {
            world,
            rigidbodies,
            collision_grid,
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

            self.rigidbodies.integrate(&mut self.world, sub_dt);

            // Do a narrow-phase collision check and generate penetration constraints
            self.collision_narrow_phase();

            self.apply_constraints(sub_dt);

            self.rigidbodies.update_velocities(&mut self.world, sub_dt);

            self.velocity_solve(sub_dt);

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

    /// Add a distance constraint between two rigidbodies.
    pub fn add_distance_constraint(
        &mut self,
        a: RigidBodyKey,
        a_attachment: Vec2<f64>,
        b: RigidBodyKey,
        b_attachment: Vec2<f64>,
        rest_dist: f64,
        compliance: f64,
    ) -> DistanceConstraintKey {
        todo!()
    }

    /// Get the calculated collision pairs with collision information.
    pub fn colliding_rigid_bodies(&mut self) -> &[(RigidBodyKey, RigidBodyKey, CollisionResponse)] {
        &self.narrow_phase_state.collisions
    }

    /// Calculate and return a 2D grid of the broad phase check.
    ///
    /// The deltatime argument is for calculating future possible positions of the bodies.
    ///
    /// This adds all rigidbodies to the grid, counts the amount of items in a bucket and drops everything.
    /// Because of that this very slow function.
    ///
    /// The amount of horizontal items of the grid is returned as the first item in the tuple.
    /// The distance between each grid element is the second element.
    pub fn broad_phase_grid(&mut self, dt: f64) -> (usize, f64, Vec<u8>) {
        self.fill_spatial_grid(dt);

        let grid = self.collision_grid.amount_map();

        // Reset the grid again so it doesn't interfere with collision detection
        self.collision_grid.clear();

        (
            SpatialGrid::<RigidBodyKey, WIDTH, HEIGHT, STEP, BUCKET, SIZE>::STEPPED_WIDTH as usize,
            STEP as f64,
            grid,
        )
    }

    /// Whether a rigidbody is still in the grid range.
    pub fn is_rigidbody_on_grid(&self, rigidbody: &RigidBodyHandle) -> bool {
        /*
        self.collision_grid
            .is_aabr_in_range(self.rigidbody(rigidbody).aabr().as_())
        */
        todo!()
    }

    /// Do a broad-phase collision pass to get possible pairs.
    ///
    /// Fills the list of broad-phase collisions.
    fn collision_broad_phase(&mut self, dt: f64) {
        self.broad_phase_collisions.clear();

        {
            puffin::profile_scope!("Insert into spatial grid");

            self.fill_spatial_grid(dt);
        }

        {
            puffin::profile_scope!("Flush spatial grid");

            // Then flush it to get the rough list of collision pairs
            self.collision_grid
                .flush_into(&mut self.broad_phase_collisions);
        }
    }

    /// Fill the spatial grid with AABR information from the rigidbodies.
    fn fill_spatial_grid(&mut self, dt: f64) {
        puffin::profile_function!();

        // First put all rigid bodies in the spatial grid
        self.predicted_aabrs(dt)
            .into_iter()
            .for_each(|(index, aabr)| self.collision_grid.store_aabr(aabr.as_(), index));
    }

    /// Do a narrow-phase collision pass to get all colliding objects.
    ///
    /// Fills the penetration constraint list and the list of collisions.
    fn collision_narrow_phase(&mut self) {
        self.narrow_phase_state.clear();

        // Narrow-phase with SAT
        /*
        for (a, b) in self.broad_phase_collisions.iter() {
            // Ignore inactive collisions
            if !self.rigidbody(*a).is_active() && !self.rigidbody(*b).is_active() {
                continue;
            }

            puffin::profile_scope!("Narrow collision");

            self.rigidbodies[*a].push_collisions(
                *a,
                &self.rigidbodies[*b],
                *b,
                &mut self.narrow_phase_state,
            );
        }

        {
            puffin::profile_scope!("Collision responses to penetration constraints");

            // Generate penetration constraint
            for (a, b, response) in self.narrow_phase_state.collisions.iter() {
                /*
                self.penetration_constraints
                    .push(PenetrationConstraint::new([*a, *b], response.clone()));
                */
            }
        }
        */
    }

    /// Debug information for all constraints.
    pub fn debug_info_constraints(&self) -> Vec<(Vec2<f64>, Vec2<f64>)> {
        puffin::profile_function!();

        /*
        self.dist_constraints
            .iter()
            .map(|(_, dist_constraint)| dist_constraint.attachments_world(&self.rigidbodies))
            .collect()
        */
        todo!()
    }

    /// Apply velocity corrections caused by friction and restitution.
    fn velocity_solve(&mut self, sub_dt: f64) {
        /*
        self.penetration_constraints
            .iter()
            .for_each(|constraint| constraint.solve_velocities(&mut self.rigidbodies, sub_dt));
        */
        // TODO
    }

    fn reset_constraints(&self) {
        // TODO
    }

    fn apply_constraints(&self, sub_dt: f64) {
        // TODO
    }

    /// Iterator over all predicted Axis-aligned bounding rectangles with a predicted future position added.
    ///
    /// WARNING: `dt` is not from the substep but from the full physics step.
    // PERF: make this an iterator
    fn predicted_aabrs(&self, dt: f64) -> Vec<(Entity, Aabr<f64>)> {
        /// How far away we predict the impulses to move us for checking the collision during the next full deltatime.
        const PREDICTED_POSITION_MULTIPLIER: f64 = 2.0;

        self.world
            .query::<(&Position, &Velocity, &Orientation, &Shape)>()
            .iter()
            .map(move |(id, (pos, vel, rot, shape))| {
                // First AABR at stationary position
                let mut aabr = shape.aabr(Iso::new(pos.0, rot.0));

                // Expand to future AABR
                aabr.expand_to_contain(shape.aabr(Iso::new(
                    pos.0 + vel.0 * PREDICTED_POSITION_MULTIPLIER * dt,
                    rot.0,
                )));

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
