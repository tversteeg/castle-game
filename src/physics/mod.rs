//! XPBD based physics engine.
//!
//! Based on: https://matthias-research.github.io/pages/publications/PBDBodies.pdf

pub mod collision;
pub mod constraint;
pub mod rigidbody;

use std::collections::HashMap;

use serde::Deserialize;
use vek::{Aabr, Vec2};

use crate::assets::Assets;

use self::{
    collision::{sat::CollisionResponse, spatial_grid::SpatialGrid},
    constraint::{
        distance::DistanceConstraint, ground::GroundConstraint, penetration::PenetrationConstraint,
        Constraint, ConstraintIndex,
    },
    rigidbody::{RigidBody, RigidBodyIndex},
};

/// Physics simulation state.
#[derive(Debug)]
pub struct Simulator<
    const WIDTH: u16,
    const HEIGHT: u16,
    const STEP: u16,
    const BUCKET: usize,
    const SIZE: usize,
> {
    /// List of all rigidbodies, accessed by index.
    rigidbodies: HashMap<RigidBodyIndex, RigidBody>,
    /// Rigid body key start.
    rigidbodies_key: RigidBodyIndex,
    /// All distance constraints.
    dist_constraints: HashMap<ConstraintIndex, DistanceConstraint>,
    /// Dist constraints body key start.
    dist_constraints_key: ConstraintIndex,
    /// All ground constraints.
    ground_constraints: HashMap<ConstraintIndex, GroundConstraint>,
    /// Ground constraints body key start.
    ground_constraints_key: ConstraintIndex,
    /// Collision grid structure.
    collision_grid: SpatialGrid<RigidBodyIndex, WIDTH, HEIGHT, STEP, BUCKET, SIZE>,
}

impl<
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const SIZE: usize,
    > Simulator<WIDTH, HEIGHT, STEP, BUCKET, SIZE>
{
    /// Create the new state.
    pub fn new() -> Self {
        let rigidbodies = HashMap::new();
        let rigidbodies_key = 0;
        let dist_constraints = HashMap::new();
        let dist_constraints_key = 0;
        let ground_constraints = HashMap::new();
        let ground_constraints_key = 0;
        let collision_grid = SpatialGrid::new();

        Self {
            rigidbodies,
            rigidbodies_key,
            dist_constraints,
            dist_constraints_key,
            ground_constraints,
            ground_constraints_key,
            collision_grid,
        }
    }

    /// Simulate a single step.
    pub fn step(&mut self, dt: f32, assets: &Assets) {
        let settings = &assets.settings().physics;
        let substeps = settings.substeps;

        // Deltatime for each sub-step
        let sub_dt = dt / substeps as f32;

        // Reset every constraint for calculating the sub-steps since they are iterative
        reset_constraints(&mut self.dist_constraints);
        reset_constraints(&mut self.ground_constraints);

        // Do a broad phase collision check to get possible colliding pairs
        let broad_phase = self.collision_broad_phase_vec();

        for _ in 0..substeps {
            // Update posititons and velocity of all rigidbodies
            self.rigidbodies
                .iter_mut()
                .for_each(|(_, rigidbody)| rigidbody.integrate(sub_dt));

            // Apply constraints for the different types
            apply_constraints(&mut self.dist_constraints, &mut self.rigidbodies, sub_dt);
            apply_constraints(&mut self.ground_constraints, &mut self.rigidbodies, sub_dt);

            // Do a narrow-phase collision check
            let collisions = self.collision_narrow_phase_vec(&broad_phase);
            // Resolve collisions into new constraints
            let mut penetration_constraints = self.handle_collisions(&collisions);
            apply_constraints_vec(&mut penetration_constraints, &mut self.rigidbodies, sub_dt);

            // Finalize velocity based on position offset
            self.rigidbodies
                .iter_mut()
                .for_each(|(_, rigidbody)| rigidbody.solve(sub_dt));
        }
    }

    /// Add a rigidbody to the simulation.
    ///
    /// Returns a rigidbody reference.
    pub fn add_rigidbody(&mut self, rigidbody: RigidBody) -> RigidBodyIndex {
        self.rigidbodies_key += 1;
        self.rigidbodies.insert(self.rigidbodies_key, rigidbody);

        self.rigidbodies_key
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

    /// Add a ground constraint for a rigidbody.
    pub fn add_ground_constraint(
        &mut self,
        rigidbody: RigidBodyIndex,
        ground_height: f32,
    ) -> ConstraintIndex {
        self.ground_constraints_key += 1;
        self.ground_constraints.insert(
            self.ground_constraints_key,
            GroundConstraint::new(rigidbody, ground_height),
        );

        self.ground_constraints_key
    }

    /// Move a rigidbody to a specific position.
    pub fn set_position(&mut self, rigidbody: RigidBodyIndex, position: Vec2<f32>) {
        self.rigidbodies
            .get_mut(&rigidbody)
            .expect("Rigid body doesn't exist anymore")
            .set_position(position, false);
    }

    /// Apply a force on a rigidbody.
    pub fn apply_force(&mut self, rigidbody: RigidBodyIndex, force: Vec2<f32>) {
        self.rigidbodies
            .get_mut(&rigidbody)
            .expect("Rigid body doesn't exist anymore")
            .apply_force(force);
    }

    /// Apply a rotational force on a rigidbody.
    pub fn apply_rotational_force(&mut self, rigidbody: RigidBodyIndex, force: f32) {
        self.rigidbodies
            .get_mut(&rigidbody)
            .expect("Rigid body doesn't exist anymore")
            .apply_rotational_force(force);
    }

    /// Reference to a rigid body.
    pub fn rigidbody(&self, rigidbody: RigidBodyIndex) -> &RigidBody {
        self.rigidbodies
            .get(&rigidbody)
            .expect("Rigid body does not exist")
    }

    /// Global position of a rigidbody.
    pub fn position(&self, rigidbody: RigidBodyIndex) -> Vec2<f32> {
        self.rigidbodies[&rigidbody].position()
    }

    /// Rotation of a rigidbody as radians.
    pub fn rotation(&self, rigidbody: RigidBodyIndex) -> f32 {
        self.rigidbodies[&rigidbody].rotation()
    }

    /// Axis-aligned bounding rectangle of a a rigidbody.
    pub fn aabr(&self, rigidbody: RigidBodyIndex) -> Aabr<f32> {
        self.rigidbodies[&rigidbody].aabr()
    }

    /// Calculate all pairs of indices for colliding rigid bodies.
    ///
    /// It's done in two steps main:
    ///
    /// Broad-phase:
    /// 1. Put all rigid body bounding rectangles in a spatial grid
    /// 2. Flush the grid again returning all colliding pairs
    ///
    /// Narrow-phase:
    /// 1. Use separating axis theorem to determine the collisions and get the impulses.
    pub fn colliding_rigid_bodies(
        &mut self,
    ) -> Vec<(RigidBodyIndex, RigidBodyIndex, CollisionResponse)> {
        // Broad phase
        let broad_phase = self.collision_broad_phase_vec();

        // Narrow phase
        self.collision_narrow_phase_vec(&broad_phase)
    }

    /// Convert collisions to a list of constraints.
    fn handle_collisions(
        &mut self,
        collisions: &[(RigidBodyIndex, RigidBodyIndex, CollisionResponse)],
    ) -> Vec<PenetrationConstraint> {
        collisions
            .iter()
            .map(|(a, b, response)| PenetrationConstraint::new([*a, *b], response.clone()))
            .collect()
    }

    /// Do a broad-phase collision pass to get possible pairs.
    ///
    /// Returns a list of pairs that might collide.
    fn collision_broad_phase_vec(&mut self) -> Vec<(RigidBodyIndex, RigidBodyIndex)> {
        // First put all rigid bodies in the spatial grid
        self.rigidbodies.iter().for_each(|(index, rigidbody)| {
            self.collision_grid.store_aabb(
                rigidbody.position().as_(),
                rigidbody.aabr().size().as_(),
                *index,
            )
        });

        // Then flush it to get the rough list of collision pairs
        self.collision_grid.flush().collect()
    }

    /// Do a narrow-phase collision pass to get all colliding objects.
    ///
    /// Returns a list of pairs with collision information.
    fn collision_narrow_phase_vec(
        &mut self,
        collision_pairs: &Vec<(RigidBodyIndex, RigidBodyIndex)>,
    ) -> Vec<(RigidBodyIndex, RigidBodyIndex, CollisionResponse)> {
        // Narrow-phase with SAT
        collision_pairs
            .iter()
            .filter_map(|(a, b)| {
                self.rigidbodies[&a]
                    .collides(&self.rigidbodies[&b])
                    .map(|response| (*a, *b, response))
            })
            .collect()
    }

    /// Debug information for all constraints.
    pub fn debug_info_constraints(&self) -> Vec<(Vec2<f32>, Vec2<f32>)> {
        self.dist_constraints
            .iter()
            .map(|(_, dist_constraint)| dist_constraint.attachments_world(&self.rigidbodies))
            .collect()
    }
}

/// Reset a type of all constraints.
fn reset_constraints<T>(constraints: &mut HashMap<ConstraintIndex, T>)
where
    T: Constraint,
{
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
    for (_, constraint) in constraints.iter_mut() {
        // Solve the constraints
        constraint.solve(rigidbodies, dt);
    }
}

/// Apply a type of constraints as an iterator to all rigidbodies.
fn apply_constraints_vec<T>(
    constraints: &mut Vec<T>,
    rigidbodies: &mut HashMap<RigidBodyIndex, RigidBody>,
    dt: f32,
) where
    T: Constraint,
{
    // Solve the constraints
    constraints
        .iter_mut()
        .for_each(|constraint| constraint.solve(rigidbodies, dt));
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
}
