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
    constraint::{Constraint, ConstraintIndex, DistanceConstraint, GroundConstraint},
    rigidbody::{RigidBody, RigidBodyIndex},
};

/// Physics simulation state.
#[derive(Debug)]
pub struct Simulator {
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
}

impl Simulator {
    /// Create the new state.
    pub fn new() -> Self {
        let rigidbodies = HashMap::new();
        let rigidbodies_key = 0;
        let dist_constraints = HashMap::new();
        let dist_constraints_key = 0;
        let ground_constraints = HashMap::new();
        let ground_constraints_key = 0;

        Self {
            rigidbodies,
            rigidbodies_key,
            dist_constraints,
            dist_constraints_key,
            ground_constraints,
            ground_constraints_key,
        }
    }

    /// Simulate a single step.
    pub fn step(&mut self, dt: f32, assets: &Assets) {
        let settings = &assets.settings().physics;
        let substeps = settings.substeps;
        let air_friction = settings.air_friction;

        // Deltatime for each sub-step
        let sub_dt = dt / substeps as f32;

        // Reset every constraint for calculating the sub-steps since they are iterative
        reset_constraints(&mut self.dist_constraints);
        reset_constraints(&mut self.ground_constraints);

        for _ in 0..substeps {
            // Update posititons and velocity of all rigidbodies
            self.rigidbodies
                .iter_mut()
                .for_each(|(_, rigidbody)| rigidbody.integrate(sub_dt));

            // Apply constraints for the different types
            apply_constraints(&mut self.dist_constraints, &mut self.rigidbodies, sub_dt);
            apply_constraints(&mut self.ground_constraints, &mut self.rigidbodies, sub_dt);

            // Finalize velocity based on position offset
            self.rigidbodies
                .iter_mut()
                .for_each(|(_, rigidbody)| rigidbody.solve(air_friction, sub_dt));
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
        rigidbodies: [RigidBodyIndex; 2],
        rest_dist: f32,
        compliance: f32,
    ) -> ConstraintIndex {
        self.dist_constraints_key += 1;
        self.dist_constraints.insert(
            self.dist_constraints_key,
            DistanceConstraint::new(rigidbodies, rest_dist, compliance),
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

    /// Apply a force on all rigidbodies.
    ///
    /// Useful for gravity.
    pub fn apply_global_force(&mut self, force: Vec2<f32>) {
        self.rigidbodies
            .iter_mut()
            .for_each(|(_, rigidbody)| rigidbody.apply_force(force));
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
}

/// Reset a type of all constraints.
fn reset_constraints<T, const RIGIDBODY_COUNT: usize>(constraints: &mut HashMap<ConstraintIndex, T>)
where
    T: Constraint<RIGIDBODY_COUNT>,
{
    for (_, constraint) in constraints.iter_mut() {
        constraint.reset();
    }
}

/// Apply a type of constraints to all rigidbodies.
fn apply_constraints<T, const RIGIDBODY_COUNT: usize>(
    constraints: &mut HashMap<ConstraintIndex, T>,
    rigidbodies: &mut HashMap<RigidBodyIndex, RigidBody>,
    dt: f32,
) where
    T: Constraint<RIGIDBODY_COUNT>,
{
    for (_, constraint) in constraints.iter_mut() {
        // Solve the constraints
        constraint.solve(rigidbodies, dt);
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
}
