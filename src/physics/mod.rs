//! XPBD based physics engine.
//!
//! Based on: https://matthias-research.github.io/pages/publications/PBDBodies.pdf

pub mod collision;
pub mod constraint;
pub mod rigidbody;

use std::rc::{Rc, Weak};

use hashbrown::HashMap;

use serde::Deserialize;
use vek::Vec2;

use self::{
    collision::{spatial_grid::SpatialGrid, CollisionResponse},
    constraint::{
        distance::DistanceConstraint, penetration::PenetrationConstraint, Constraint,
        ConstraintIndex,
    },
    rigidbody::{RigidBody, RigidBodyIndex},
};

/// Physics simulation state.
pub struct Physics<
    const WIDTH: u16,
    const HEIGHT: u16,
    const STEP: u16,
    const BUCKET: usize,
    const SIZE: usize,
> {
    /// List of all rigidbodies, accessed by index.
    rigidbodies: HashMap<RigidBodyIndex, RigidBody>,
    /// List of references to the rigidbody handles so that when a handle is dropped from anywhere we can also destroy the rigidbody.
    rigidbody_references: Vec<(Weak<RigidBodyIndex>, RigidBodyIndex)>,
    /// Rigid body key start.
    rigidbodies_key: RigidBodyIndex,
    /// All distance constraints.
    dist_constraints: HashMap<ConstraintIndex, DistanceConstraint>,
    /// Dist constraints body key start.
    dist_constraints_key: ConstraintIndex,
    /// All penetration constraints.
    ///
    /// This is a vec that's cleared multiple times per step.
    penetration_constraints: Vec<PenetrationConstraint>,
    /// Collision grid structure.
    collision_grid: SpatialGrid<RigidBodyIndex, WIDTH, HEIGHT, STEP, BUCKET, SIZE>,
    /// Cache of broad phase collisions.
    ///
    /// This is a performance optimization so the vector doesn't have to be allocated every step.
    broad_phase_collisions: Vec<(RigidBodyIndex, RigidBodyIndex)>,
    /// Cache of narrow phase collisions.
    ///
    /// This is a performance optimization so the vector doesn't have to be allocated many times every step.
    narrow_phase_collisions: Vec<(RigidBodyIndex, RigidBodyIndex, CollisionResponse)>,
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
        let rigidbodies = HashMap::new();
        let rigidbodies_key = 0;
        let rigidbody_references = Vec::new();
        let dist_constraints = HashMap::new();
        let dist_constraints_key = 0;
        let penetration_constraints = Vec::new();
        let collision_grid = SpatialGrid::new();
        let broad_phase_collisions = Vec::with_capacity(16);
        let narrow_phase_collisions = Vec::with_capacity(16);

        Self {
            rigidbodies,
            rigidbodies_key,
            rigidbody_references,
            dist_constraints,
            dist_constraints_key,
            penetration_constraints,
            collision_grid,
            broad_phase_collisions,
            narrow_phase_collisions,
        }
    }

    /// Simulate a single step.
    pub fn step(&mut self, dt: f32) {
        puffin::profile_function!(format!("{dt}"));

        let settings = &crate::settings().physics;
        let substeps = settings.substeps;

        // Deltatime for each sub-step
        let sub_dt = dt / substeps as f32;

        {
            puffin::profile_scope!("Remove dropped rigidbodies");

            // Destroy every rigidbody handle that has no references anymore
            self.destroy_dropped_rigidbodies();
        }

        {
            puffin::profile_scope!("Reset constraints");
            // Reset every constraint for calculating the sub-steps since they are iterative
            reset_constraints(&mut self.dist_constraints);
        }

        {
            puffin::profile_scope!("Broad phase collision detection");
            // Do a broad phase collision check to get possible colliding pairs
            self.collision_broad_phase(dt)
        }

        for _ in 0..substeps {
            puffin::profile_scope!("Substep");

            {
                puffin::profile_scope!("Integrate rigidbodies");

                // Update posititons and velocity of all rigidbodies
                self.rigidbodies
                    .iter_mut()
                    .for_each(|(_, rigidbody)| rigidbody.integrate(sub_dt));
            }

            {
                puffin::profile_scope!("Narrow phase collision detection");

                // Do a narrow-phase collision check and generate penetration constraints
                self.collision_narrow_phase();
            }

            {
                puffin::profile_scope!("Apply penetration constraints");

                // Apply the generated penetration constraints
                apply_constraints_vec(
                    &mut self.penetration_constraints,
                    &mut self.rigidbodies,
                    sub_dt,
                );
            }

            {
                puffin::profile_scope!("Apply distance constraints");
                // Apply constraints for the different types
                apply_constraints(&mut self.dist_constraints, &mut self.rigidbodies, sub_dt);
            }

            {
                puffin::profile_scope!("Update velocities");
                // Finalize velocity based on position offset
                self.rigidbodies
                    .iter_mut()
                    .for_each(|(_, rigidbody)| rigidbody.update_velocities(sub_dt));
            }

            {
                puffin::profile_scope!("Solve velocities");
                self.velocity_solve(sub_dt);
            }

            {
                puffin::profile_scope!("Apply translations");
                // Finalize velocity based on position offset
                self.rigidbodies
                    .iter_mut()
                    .for_each(|(_, rigidbody)| rigidbody.apply_translation());
            }
        }

        {
            puffin::profile_scope!("Mark sleeping");
            // Finalize velocity based on position offset
            self.rigidbodies
                .iter_mut()
                .for_each(|(_, rigidbody)| rigidbody.mark_sleeping(dt));
        }
    }

    /// Remove every rigidbody.
    pub fn reset(&mut self) {
        self.rigidbodies.clear();
        self.dist_constraints.clear();
    }

    /// Add a rigidbody to the simulation.
    ///
    /// Returns a rigidbody reference.
    pub fn add_rigidbody(&mut self, rigidbody: RigidBody) -> RigidBodyHandle {
        self.rigidbodies_key += 1;
        self.rigidbodies.insert(self.rigidbodies_key, rigidbody);

        let handle = RigidBodyHandle(Rc::new(self.rigidbodies_key));

        // Store a weak reference to the handle so we can destroy it when the handle is dropped
        self.rigidbody_references
            .push((handle.downgrade(), self.rigidbodies_key));

        handle
    }

    /// Add a distance constraint between two rigidbodies.
    pub fn add_distance_constraint(
        &mut self,
        a: RigidBodyIndex,
        a_attachment: Vec2<f32>,
        b: RigidBodyIndex,
        b_attachment: Vec2<f32>,
        rest_dist: f32,
        compliance: f32,
    ) -> ConstraintIndex {
        self.dist_constraints_key += 1;
        self.dist_constraints.insert(
            self.dist_constraints_key,
            DistanceConstraint::new(a, a_attachment, b, b_attachment, rest_dist, compliance),
        );

        self.dist_constraints_key
    }

    /// Check whether a rigidbody is still alive.
    pub fn has_rigidbody(&self, rigidbody: RigidBodyIndex) -> bool {
        self.rigidbodies.contains_key(&rigidbody)
    }

    /// Reference to a rigid body.
    pub fn rigidbody(&self, rigidbody: RigidBodyIndex) -> &RigidBody {
        puffin::profile_function!();

        self.rigidbodies
            .get(&rigidbody)
            .expect("Rigid body does not exist")
    }

    /// Mutable reference to a rigid body.
    pub fn rigidbody_mut(&mut self, rigidbody: RigidBodyIndex) -> &mut RigidBody {
        puffin::profile_function!();

        self.rigidbodies
            .get_mut(&rigidbody)
            .expect("Rigid body does not exist")
    }

    /// Reference to all rigid bodies.
    pub fn rigidbody_map(&self) -> &HashMap<RigidBodyIndex, RigidBody> {
        puffin::profile_function!();

        &self.rigidbodies
    }

    /// Get the calculated collision pairs with collision information.
    pub fn colliding_rigid_bodies(
        &mut self,
    ) -> &[(RigidBodyIndex, RigidBodyIndex, CollisionResponse)] {
        puffin::profile_function!();

        &self.narrow_phase_collisions
    }

    /// Do a broad-phase collision pass to get possible pairs.
    ///
    /// Fills the list of broad-phase collisions.
    fn collision_broad_phase(&mut self, dt: f32) {
        puffin::profile_function!();

        {
            puffin::profile_scope!("Clear vector");

            self.broad_phase_collisions.clear();
        }

        {
            puffin::profile_scope!("Insert into spatial grid");

            // First put all rigid bodies in the spatial grid
            self.rigidbodies.iter().for_each(|(index, rigidbody)| {
                self.collision_grid
                    .store_aabr(rigidbody.predicted_aabr(dt).as_(), *index)
            });
        }

        {
            puffin::profile_scope!("Flush spatial grid");

            // Then flush it to get the rough list of collision pairs
            self.collision_grid
                .flush_into(&mut self.broad_phase_collisions);
        }
    }

    /// Do a narrow-phase collision pass to get all colliding objects.
    ///
    /// Fills the penetration constraint list and the list of collisions.
    fn collision_narrow_phase(&mut self) {
        puffin::profile_function!();

        {
            puffin::profile_scope!("Clear vectors");

            self.narrow_phase_collisions.clear();
            self.penetration_constraints.clear();
        }

        // Narrow-phase with SAT
        for (a, b) in self.broad_phase_collisions.iter() {
            // Ignore inactive collisions
            if !self.rigidbody(*a).is_active() && !self.rigidbody(*b).is_active() {
                continue;
            }

            puffin::profile_scope!("Narrow collision");

            self.rigidbodies[a].push_collisions(
                *a,
                &self.rigidbodies[b],
                *b,
                &mut self.narrow_phase_collisions,
            );
        }

        {
            puffin::profile_scope!("Collision responses to penetration constraints");

            // Generate penetration constraint
            for (a, b, response) in self.narrow_phase_collisions.iter() {
                self.penetration_constraints
                    .push(PenetrationConstraint::new([*a, *b], response.clone()));
            }
        }
    }

    /// Debug information for all constraints.
    pub fn debug_info_constraints(&self) -> Vec<(Vec2<f32>, Vec2<f32>)> {
        puffin::profile_function!();

        self.dist_constraints
            .iter()
            .map(|(_, dist_constraint)| dist_constraint.attachments_world(&self.rigidbodies))
            .collect()
    }

    /// Apply velocity corrections caused by friction and restitution.
    fn velocity_solve(&mut self, sub_dt: f32) {
        self.penetration_constraints
            .iter()
            .for_each(|constraint| constraint.solve_velocities(&mut self.rigidbodies, sub_dt));
    }

    /// Destroy every rigidbody where the handle is dropped.
    fn destroy_dropped_rigidbodies(&mut self) {
        self.rigidbody_references.retain(|(reference, rigidbody)| {
            if reference.strong_count() == 0 {
                // Remove the rigidbody
                self.rigidbodies.remove(rigidbody);

                false
            } else {
                true
            }
        });
    }
}

/// Handle that will destroy the rigidbody when it's dropped.
#[derive(Debug, Clone)]
pub struct RigidBodyHandle(Rc<RigidBodyIndex>);

impl RigidBodyHandle {
    /// Get a reference to the rigidbody this is a handle from.
    pub fn rigidbody<
        'a,
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const SIZE: usize,
    >(
        &self,
        physics: &'a Physics<WIDTH, HEIGHT, STEP, BUCKET, SIZE>,
    ) -> &'a RigidBody {
        physics.rigidbody(*self.0)
    }

    /// Get a mutable reference to the rigidbody this is a handle from.
    pub fn rigidbody_mut<
        'a,
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const SIZE: usize,
    >(
        &self,
        physics: &'a mut Physics<WIDTH, HEIGHT, STEP, BUCKET, SIZE>,
    ) -> &'a mut RigidBody {
        physics.rigidbody_mut(*self.0)
    }

    /// Create a weak reference to the rigidbody.
    pub fn downgrade(&self) -> Weak<RigidBodyIndex> {
        Rc::downgrade(&self.0)
    }
}

/// Physics settings loaded from a file so it's easier to change them with hot-reloading.
#[derive(Deserialize)]
pub struct Settings {
    /// How many substeps are taken in a single step.
    pub substeps: u8,
    /// Gravity applied every frame.
    pub gravity: f32,
    /// Damping applied to the velocity every timestep.
    pub air_friction: f32,
    /// Dampling applied to the rotation every timestep.
    pub rotation_friction: f32,
}

/// Reset a type of all constraints.
fn reset_constraints<T>(constraints: &mut HashMap<ConstraintIndex, T>)
where
    T: Constraint,
{
    puffin::profile_function!();

    for (_, constraint) in constraints.iter_mut() {
        constraint.reset();
    }
}

/// Apply a type of constraints to all rigidbodies.
fn apply_constraints<T>(
    constraints: &mut HashMap<ConstraintIndex, T>,
    rigidbodies: &mut HashMap<RigidBodyIndex, RigidBody>,
    dt: f32,
) where
    T: Constraint,
{
    puffin::profile_function!();

    for (_, constraint) in constraints.iter_mut() {
        // Solve the constraints
        constraint.solve(rigidbodies, dt);
    }
}

/// Apply a type of constraints as an iterator to all rigidbodies.
fn apply_constraints_vec<T>(
    constraints: &mut [T],
    rigidbodies: &mut HashMap<RigidBodyIndex, RigidBody>,
    dt: f32,
) where
    T: Constraint,
{
    puffin::profile_function!();

    // Solve the constraints
    constraints
        .iter_mut()
        .for_each(|constraint| constraint.solve(rigidbodies, dt));
}
