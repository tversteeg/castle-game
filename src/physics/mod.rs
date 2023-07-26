//! XPBD based physics engine.
//!
//! Based on: https://matthias-research.github.io/pages/publications/PBDBodies.pdf

pub mod constraint;
pub mod rigidbody;

use serde::Deserialize;
use vek::Vec2;

use crate::assets::Assets;

use self::{
    constraint::{Constraint, DistanceConstraint},
    rigidbody::{RigidBody, RigidBodyIndex},
};

/// Physics simulation state.
#[derive(Debug)]
pub struct Simulator {
    /// List of all rigidbodies, accessed by index.
    rigidbodies: Vec<RigidBody>,
    /// All distance constraints.
    dist_constraints: Vec<DistanceConstraint>,
}

impl Simulator {
    /// Create the new state.
    pub fn new() -> Self {
        let rigidbodies = Vec::new();
        let dist_constraints = Vec::new();

        Self {
            rigidbodies,
            dist_constraints,
        }
    }

    /// Simulate a single step.
    pub fn step(&mut self, dt: f64, assets: &Assets) {
        let settings = &assets.settings().physics;
        let substeps = settings.substeps;
        let air_friction = settings.air_friction;

        // Deltatime for each sub-step
        let sub_dt = dt / substeps as f64;

        // Reset every constraint for calculating the sub-steps since they are iterative
        reset_constraints(&mut self.dist_constraints);

        for _ in 0..substeps {
            // Update posititons and velocity of all rigidbodies
            self.rigidbodies
                .iter_mut()
                .for_each(|rigidbody| rigidbody.step(sub_dt));

            // Apply constraints for the different types
            apply_constraints(&mut self.dist_constraints, &mut self.rigidbodies, sub_dt);

            // Finalize velocity based on position offset
            self.rigidbodies
                .iter_mut()
                .for_each(|rigidbody| rigidbody.step_finalize(air_friction, sub_dt));
        }
    }

    /// Add a rigidbody to the simulation.
    ///
    /// Returns a rigidbody reference.
    pub fn add_rigidbody(&mut self, rigidbody: RigidBody) -> RigidBodyIndex {
        self.rigidbodies.push(rigidbody);

        self.rigidbodies.len() as u32 - 1
    }

    /// Add a distance constraint between two rigidbodies.
    pub fn add_distance_constraint(
        &mut self,
        rigidbodies: [RigidBodyIndex; 2],
        rest_dist: f64,
        compliance: f64,
    ) {
        self.dist_constraints
            .push(DistanceConstraint::new(rigidbodies, rest_dist, compliance));
    }

    /// Move a rigidbody to a specific position.
    pub fn set_position(&mut self, rigidbody: RigidBodyIndex, position: Vec2<f64>) {
        self.rigidbodies[rigidbody as usize].set_position(position, false);
    }

    /// Apply a force on a rigidbody.
    pub fn apply_force(&mut self, rigidbody: RigidBodyIndex, force: Vec2<f64>) {
        self.rigidbodies[rigidbody as usize].apply_force(force);
    }

    /// Apply a force on all rigidbodies.
    ///
    /// Useful for gravity.
    pub fn apply_global_force(&mut self, force: Vec2<f64>) {
        self.rigidbodies
            .iter_mut()
            .for_each(|rigidbody| rigidbody.apply_force(force));
    }

    /// Global position of a rigidbody.
    pub fn position(&self, rigidbody: RigidBodyIndex) -> Vec2<f64> {
        self.rigidbodies[rigidbody as usize].position()
    }

    /// Rotation of a rigidbody as radians.
    pub fn rotation(&self, rigidbody: RigidBodyIndex) -> f64 {
        self.rigidbodies[rigidbody as usize].rotation()
    }
}

/// Reset a type of all constraints.
fn reset_constraints<T, const RIGIDBODY_COUNT: usize>(constraints: &mut [T])
where
    T: Constraint<RIGIDBODY_COUNT>,
{
    for constraint in constraints.iter_mut() {
        constraint.reset();
    }
}

/// Apply a type of constraints to all rigidbodies.
fn apply_constraints<T, const RIGIDBODY_COUNT: usize>(
    constraints: &mut [T],
    rigidbodies: &mut [RigidBody],
    dt: f64,
) where
    T: Constraint<RIGIDBODY_COUNT>,
{
    for constraint in constraints.iter_mut() {
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
    pub gravity: f64,
    /// Damping applied to the velocity every timestep.
    pub air_friction: f64,
}
