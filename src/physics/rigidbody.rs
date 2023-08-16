use std::rc::{Rc, Weak};

use hecs::{Bundle, ComponentRef, Entity, World};
use vek::{Aabr, Vec2};

use crate::math::{Iso, Rotation};

use super::{
    collision::{shape::Shape, CollisionResponse, CollisionState},
    Physics,
};

/// How far away we predict the impulses to move us for checking the collision during the next full deltatime.
const PREDICTED_POSITION_MULTIPLIER: f64 = 2.0;

/// Represents any physics object that can have constraints applied.
#[derive(Clone, Default)]
pub struct RigidBody {
    /// Global position.
    pos: Vec2<f64>,
    /// Previous position.
    prev_pos: Vec2<f64>,
    /// Global offset that will be added to the body.
    translation: Vec2<f64>,
    /// Velocity.
    vel: Vec2<f64>,
    /// Previous velocity.
    prev_vel: Vec2<f64>,
    /// Orientation in radians.
    rot: Rotation,
    /// Previous orientation.
    prev_rot: Rotation,
    /// Angular velocity.
    ang_vel: f64,
    /// Previous angular velocity.
    prev_ang_vel: f64,
    /// Inertia tensor, corresponds to mass in rotational terms.
    ///
    /// Torque needed for an angular acceleration.
    inertia: f64,
    /// Linear damping.
    lin_damping: f64,
    /// Angular damping.
    ang_damping: f64,
    /// External forces.
    ext_force: Vec2<f64>,
    // External torque.
    ext_torque: f64,
    /// Inverse of the mass.
    inv_mass: f64,
    /// Friction coefficient, for now there's no difference between dynamic and static friction.
    friction: f64,
    /// Restitution coefficient, how bouncy collisions are.
    restitution: f64,
    /// Collision shape.
    shape: Shape,
    /// If > 0 we are sleeping, which means we don't have to calculate a lot of steps.
    ///
    /// After a certain time the velocity and position will be set to zero.
    time_sleeping: f64,
}

impl RigidBody {
    /// Construct a new rigidbody without movements.
    ///
    /// Gravity is applied as an external force.
    pub fn new<S>(pos: Vec2<f64>, shape: S) -> Self
    where
        S: Into<Shape>,
    {
        let settings = crate::settings();

        Self::new_external_force(
            pos,
            Vec2::new(0.0, settings.physics.gravity),
            settings.physics.air_friction,
            settings.physics.rotation_friction,
            1.0,
            shape,
        )
    }

    /// Construct a new rigidbody with acceleration.
    pub fn new_external_force<S>(
        pos: Vec2<f64>,
        ext_force: Vec2<f64>,
        lin_damping: f64,
        ang_damping: f64,
        density: f64,
        shape: S,
    ) -> Self
    where
        S: Into<Shape>,
    {
        let prev_pos = pos;
        let vel = Vec2::default();
        let prev_vel = vel;
        let ang_vel = 0.0;
        let prev_ang_vel = ang_vel;
        let rot = Rotation::default();
        let prev_rot = rot;
        let ext_torque = 0.0;
        let friction = 0.3;
        let restitution = 0.3;
        let translation = Vec2::zero();
        let time_sleeping = 0.0;
        let shape = shape.into();
        let mass_properties = shape.mass_properties(density);
        let inv_mass = mass_properties.mass().recip();
        let inertia = mass_properties.principal_inertia();

        Self {
            pos,
            prev_pos,
            translation,
            vel,
            prev_vel,
            ext_force,
            ext_torque,
            lin_damping,
            inv_mass,
            inertia,
            rot,
            prev_rot,
            ang_vel,
            prev_ang_vel,
            ang_damping,
            shape,
            friction,
            restitution,
            time_sleeping,
        }
    }

    /// Construct a fixed rigidbody with infinite mass and no gravity.
    pub fn new_fixed<S>(pos: Vec2<f64>, shape: S) -> Self
    where
        S: Into<Shape>,
    {
        let inv_mass = 0.0;

        let prev_pos = pos;
        let translation = Vec2::zero();
        let vel = Vec2::zero();
        let prev_vel = vel;
        let rot = Rotation::default();
        let prev_rot = Rotation::default();
        let ang_vel = 0.0;
        let prev_ang_vel = ang_vel;
        let lin_damping = 0.0;
        let ang_damping = 0.0;
        let ext_force = Vec2::zero();
        let ext_torque = 0.0;
        let friction = 0.5;
        let restitution = 0.2;
        let time_sleeping = 0.0;
        let shape = shape.into();
        let inertia = 1.0;

        Self {
            pos,
            shape,
            prev_pos,
            translation,
            prev_vel,
            vel,
            rot,
            prev_rot,
            ang_vel,
            prev_ang_vel,
            inertia,
            lin_damping,
            ang_damping,
            ext_force,
            ext_torque,
            inv_mass,
            friction,
            restitution,
            time_sleeping,
        }
    }

    /// Apply velocity after creating a rigidbody.
    pub fn with_velocity(mut self, velocity: Vec2<f64>) -> Self {
        self.vel = velocity;
        self.prev_vel = velocity;

        self
    }

    /// Set the density.
    ///
    /// This will change the mass and inertia.
    pub fn with_density(mut self, density: f64) -> Self {
        let mass_properties = self.shape.mass_properties(density);
        self.inv_mass = mass_properties.mass().recip();
        self.inertia = mass_properties.principal_inertia();

        self
    }

    /// Set the initial rotation.
    ///
    /// It's smart to set this to the velocity direction for flying objects with stabilizing torque.
    pub fn with_rotation(mut self, rotation: Rotation) -> Self {
        self.rot = rotation;
        self.prev_rot = rotation;

        self
    }

    /// Set the dynamic and static frictions.
    pub fn with_friction(mut self, friction: f64) -> Self {
        self.friction = friction;

        self
    }

    /// Set the restitution.
    pub fn with_restitution(mut self, restitution: f64) -> Self {
        self.restitution = restitution;

        self
    }

    /// Set the linear damping.
    pub fn with_linear_damping(mut self, linear_damping: f64) -> Self {
        self.lin_damping = linear_damping;

        self
    }

    /// Set the angular damping.
    pub fn with_angular_damping(mut self, angular_damping: f64) -> Self {
        self.ang_damping = angular_damping;

        self
    }

    /// Perform a single (sub-)step with a deltatime.
    #[inline]
    pub fn integrate(&mut self, dt: f64) {
        if !self.is_active() {
            return;
        }

        // Position update
        self.prev_pos = self.pos;

        // Apply damping if applicable
        if self.lin_damping != 1.0 {
            self.vel *= 1.0 / (1.0 + dt * self.lin_damping);
        }

        // Apply external forces
        self.vel += (dt * self.ext_force) / self.inv_mass.recip();
        self.translation += dt * self.vel;

        // Rotation update
        self.prev_rot = self.rot;

        // Apply damping if applicable
        if self.ang_damping != 1.0 {
            self.ang_vel *= 1.0 / (1.0 + dt * self.ang_damping);
        }

        self.ang_vel += dt * self.inverse_inertia() * self.ext_torque;
        self.rot += dt * self.ang_vel;
    }

    /// Add velocities.
    #[inline]
    pub fn update_velocities(&mut self, dt: f64) {
        self.prev_vel = self.vel;
        self.vel = (self.pos - self.prev_pos + self.translation) / dt;

        self.prev_ang_vel = self.ang_vel;
        self.ang_vel = (self.rot - self.prev_rot).to_radians() / dt;
    }

    /// Apply translations to the position.
    #[inline]
    pub fn apply_translation(&mut self) {
        if !self.is_active() {
            return;
        }

        self.pos += self.translation;
        self.translation = Vec2::zero();
    }

    /// Apply a force by moving the position, which will trigger velocity increments.
    #[inline]
    pub fn apply_force(&mut self, force: Vec2<f64>) {
        self.translation += force;
    }

    /// Apply a rotational force in radians.
    #[inline]
    pub fn apply_rotational_force(&mut self, force: f64) {
        self.rot += force;
    }

    /// Apply torque from an external source.
    #[inline]
    pub fn apply_torque(&mut self, torque: f64) {
        self.ext_torque += torque;
    }

    /// Set global position.
    pub fn set_position(&mut self, pos: Vec2<f64>, force: bool) {
        self.pos = pos;
        if !force {
            self.prev_pos = pos;
            self.translation = Vec2::zero();
            self.vel = Vec2::zero();
        }
    }

    /// Set the rigidbody to sleeping if the velocities are below the treshold.
    pub fn mark_sleeping(&mut self, dt: f64) {
        puffin::profile_scope!("Mark sleeping");

        if self.is_static() {
            return;
        }

        // TODO: make these values configurable
        if self.vel.magnitude_squared() > 1.0 || self.ang_vel.abs() > 1.0 {
            self.time_sleeping = 0.0;
        } else if self.time_sleeping < 0.5 {
            self.time_sleeping += dt;
        } else {
            // Set the velocities to zero to prevent jittering
            self.vel = Vec2::zero();
            self.ang_vel = 0.0;
        }
    }

    /// Global position.
    #[inline]
    pub fn position(&self) -> Vec2<f64> {
        self.pos + self.translation
    }

    /// Global linear velocity.
    #[inline]
    pub fn velocity(&self) -> Vec2<f64> {
        self.vel
    }

    /// Global angular velocity.
    #[inline]
    pub fn angular_velocity(&self) -> f64 {
        self.ang_vel
    }

    /// Global position with rotation.
    #[inline]
    pub fn iso(&self) -> Iso {
        puffin::profile_scope!("Iso");

        Iso::new(self.position(), self.rot)
    }

    /// Orientation of the body.
    pub fn rotation(&self) -> Rotation {
        self.rot
    }

    /// Calculate generalized inverse mass at a relative point along the normal vector.
    #[inline]
    pub fn inverse_mass_at_relative_point(&self, point: Vec2<f64>, normal: Vec2<f64>) -> f64 {
        puffin::profile_scope!("Inverse mass at relative point");

        if self.is_static() {
            return 0.0;
        }

        // Perpendicular dot product of `point` with `normal`
        let perp_dot = (point.x * normal.y) - (point.y * normal.x);

        self.inv_mass + self.inverse_inertia() * perp_dot.powi(2)
    }

    /// Calculate the update in rotation when a position change is applied at a specific point.
    pub fn delta_rotation_at_point(&self, point: Vec2<f64>, impulse: Vec2<f64>) -> f64 {
        puffin::profile_scope!("Delta rotation at point");

        // Perpendicular dot product of `point` with `impulse`
        let perp_dot = (point.x * impulse.y) - (point.y * impulse.x);

        self.inverse_inertia() * perp_dot
    }

    /// Apply a positional impulse at a point.
    ///
    // TODO: can we remove the sign by directly negating the impulse?
    #[inline]
    pub fn apply_positional_impulse(
        &mut self,
        positional_impulse: Vec2<f64>,
        point: Vec2<f64>,
        sign: f64,
    ) {
        puffin::profile_scope!("Apply positional impulse");

        if self.is_static() {
            // Ignore when we're a static body
            return;
        }

        self.apply_force(sign * positional_impulse * self.inv_mass);

        // Change rotation of body
        self.apply_rotational_force(sign * self.delta_rotation_at_point(point, positional_impulse));
    }

    /// Apply a velocity change at a point.
    #[inline]
    pub fn apply_velocity_impulse(
        &mut self,
        velocity_impulse: Vec2<f64>,
        point: Vec2<f64>,
        sign: f64,
    ) {
        puffin::profile_scope!("Apply velocity impulse");

        if self.is_static() {
            // Ignore when we're a static body
            return;
        }

        self.vel += sign * velocity_impulse * self.inv_mass;
        self.ang_vel += sign * self.delta_rotation_at_point(point, velocity_impulse);
    }

    /// Calculate the contact velocity based on a local relative rotated point.
    #[inline]
    pub fn contact_velocity(&self, point: Vec2<f64>) -> Vec2<f64> {
        // Perpendicular
        let perp = Vec2::new(-point.y, point.x);

        self.vel + self.ang_vel * perp
    }

    /// Calculate the contact velocity based on a local relative rotated point with the previous velocities.
    #[inline]
    pub fn prev_contact_velocity(&self, point: Vec2<f64>) -> Vec2<f64> {
        // Perpendicular
        let perp = Vec2::new(-point.y, point.x);

        self.prev_vel + self.prev_ang_vel * perp
    }

    /// Delta position of a point.
    #[inline]
    pub fn relative_motion_at_point(&self, point: Vec2<f64>) -> Vec2<f64> {
        self.pos - self.prev_pos + self.translation + point - self.prev_rot.rotate(point)
    }

    /// Inverse of the inertia tensor.
    #[inline]
    pub fn inverse_inertia(&self) -> f64 {
        self.inertia.recip()
    }

    /// Axis-aligned bounding rectangle.
    #[inline]
    pub fn aabr(&self) -> Aabr<f64> {
        self.shape.aabr(self.iso())
    }

    /// Vertices for the body.
    pub fn vertices(&self) -> Vec<Vec2<f64>> {
        self.shape.vertices(self.iso())
    }

    /// Axis-aligned bounding rectangle with a predicted future position added.
    ///
    /// WARNING: `dt` is not from the substep but from the full physics step.
    #[inline]
    pub fn predicted_aabr(&self, dt: f64) -> Aabr<f64> {
        puffin::profile_scope!("Predicted AABR");

        // If we are static or sleeping there's nothing to predict
        if !self.is_active() {
            return self.aabr();
        }

        // Start with the future aabr
        let mut aabr = self.shape.aabr(Iso::new(
            self.position() + self.vel * PREDICTED_POSITION_MULTIPLIER * dt,
            self.rot,
        ));

        // Add the current aabr
        aabr.expand_to_contain(self.aabr());

        aabr
    }

    /// Check if it collides with another rigidbody.
    ///
    /// This function is very inefficient, use [`Self::push_collisions`].
    pub fn collides(&self, other: &RigidBody) -> Vec<CollisionResponse> {
        self.shape.collides(self.iso(), &other.shape, other.iso())
    }

    /// Check if it collides with another rigidbody.
    ///
    /// Pushes to a buffer with collision information when it does.
    #[inline]
    pub fn push_collisions<K>(
        &self,
        a_data: K,
        b: &RigidBody,
        b_data: K,
        state: &mut CollisionState<K>,
    ) where
        K: Clone,
    {
        self.shape
            .push_collisions(self.iso(), a_data, &b.shape, b.iso(), b_data, state);
    }

    /// Rotate a point in local space.
    #[inline]
    pub fn rotate(&self, point: Vec2<f64>) -> Vec2<f64> {
        puffin::profile_scope!("Rotate in local space");

        self.rot.rotate(point)
    }

    /// Calculate the world position of a relative point on this body without rotation in mind.
    #[inline]
    pub fn local_to_world(&self, point: Vec2<f64>) -> Vec2<f64> {
        puffin::profile_scope!("Local coordinates to world");

        // Then translate it to the position
        self.position() + self.rotate(point)
    }

    /// Whether this rigidbody doesn't move and has infinite mass.
    #[inline]
    pub fn is_static(&self) -> bool {
        self.inv_mass == 0.0
    }

    /// Whether the rigidbody is in a sleeping state.
    #[inline]
    pub fn is_sleeping(&self) -> bool {
        self.time_sleeping >= 0.5
    }

    /// Whether this is an active rigid body, means it's not sleeping and not static.
    #[inline]
    pub fn is_active(&self) -> bool {
        !self.is_static() && !self.is_sleeping()
    }

    /// Friction that needs to be overcome before resting objects start sliding.
    #[inline]
    pub fn static_friction(&self) -> f64 {
        self.friction
    }

    /// Friction that's applied to dynamic moving object after static friction has been overcome.
    #[inline]
    pub fn dynamic_friction(&self) -> f64 {
        self.friction
    }

    /// Combine the static frictions between this and another body.
    #[inline]
    pub fn combine_static_frictions(&self, other: &Self) -> f64 {
        (self.static_friction() + other.static_friction()) / 2.0
    }

    /// Combine the dynamic frictions between this and another body.
    #[inline]
    pub fn combine_dynamic_frictions(&self, other: &Self) -> f64 {
        (self.dynamic_friction() + other.dynamic_friction()) / 2.0
    }

    /// Combine the restitutions between this and another body.
    #[inline]
    pub fn combine_restitutions(&self, other: &Self) -> f64 {
        (self.restitution + other.restitution) / 2.0
    }

    /// Current direction the body is moving in.
    #[inline]
    pub fn direction(&self) -> Vec2<f64> {
        (self.pos - self.prev_pos).normalized()
    }
}

/// Rigidbody builder types.
enum RigidBodyBuilderType {
    Dynamic,
    Kinetic,
    Static,
}

/// Create a new rigidbody.
pub struct RigidBodyBuilder {
    position: Vec2<f64>,
    velocity: Vec2<f64>,
    linear_damping: f64,
    orientation: Rotation,
    angular_velocity: f64,
    angular_damping: f64,
    density: f64,
    friction: f64,
    restitution: f64,
    collider: Shape,
    body_type: RigidBodyBuilderType,
}

impl RigidBodyBuilder {
    /// Create a new dynamic rigidbody.
    ///
    /// Dynamic means it's influenced by all forces and its position is updated accordingly.
    /// Good examples for it are projectiles and shrapnel.
    #[must_use]
    pub fn new(position: Vec2<f64>) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Create a new kinetic rigidbody.
    ///
    /// Kinetic means it's influenced by all forces but its position is not updated.
    /// A kinetic body can still have mass and should handle collision events.
    /// Good examples for it are player controllers.
    #[must_use]
    pub fn new_kinetic(position: Vec2<f64>) -> Self {
        let body_type = RigidBodyBuilderType::Kinetic;

        Self {
            position,
            body_type,
            ..Default::default()
        }
    }

    /// Create a new static rigidbody.
    ///
    /// Static means it's not influenced by any forces and has infinite mass.
    /// Good example for it is the ground.
    #[must_use]
    pub fn new_static(position: Vec2<f64>) -> Self {
        let body_type = RigidBodyBuilderType::Static;

        Self {
            position,
            body_type,
            ..Default::default()
        }
    }

    /// Set the collider from a shape.
    #[must_use]
    pub fn with_collider(mut self, collider: Shape) -> Self {
        self.collider = collider;

        self
    }

    /// Set the world-space initial linear velocity.
    #[must_use]
    pub fn with_velocity(mut self, velocity: Vec2<f64>) -> Self {
        self.velocity = velocity;

        self
    }

    /// Set the linear damping.
    ///
    /// This can be used to simulate things like air friction.
    #[must_use]
    pub fn with_linear_damping(mut self, linear_damping: f64) -> Self {
        self.linear_damping = linear_damping;

        self
    }

    /// Set the initial orientation/rotation.
    #[must_use]
    pub fn with_orientation<R>(mut self, orientation: R) -> Self
    where
        R: Into<Rotation>,
    {
        self.orientation = orientation.into();

        self
    }

    /// Set the initial orientation/rotation pointing towards the direction.
    ///
    /// Assumes the direction is normalized.
    #[must_use]
    pub fn with_orientation_from_direction(mut self, direction: Vec2<f64>) -> Self {
        self.orientation = Rotation::from_direction(direction);

        self
    }

    /// Set the world-space initial angular velocity.
    ///
    /// This is how many radians per time unit the orientation changes (rotates).
    #[must_use]
    pub fn with_angular_velocity(mut self, angular_velocity: f64) -> Self {
        self.angular_velocity = angular_velocity;

        self
    }

    /// Set the angular damping.
    #[must_use]
    pub fn with_angular_damping(mut self, angular_damping: f64) -> Self {
        self.angular_damping = angular_damping;

        self
    }

    /// Set the density.
    ///
    /// Density is mass per 1x1 surface of the collider object.
    /// From this value inertia and mass will be calculated based on the collider shape.
    #[must_use]
    pub fn with_density(mut self, density: f64) -> Self {
        self.density = density;

        self
    }

    /// Set the dynamic and static friction.
    ///
    /// Static friction is how much friction is needed to overcome before an object starts moving.
    /// Dynamic friction is how much friction is applied when colliding to another object.
    #[must_use]
    pub fn with_friction(mut self, friction: f64) -> Self {
        self.friction = friction;

        self
    }

    /// Set the restitution.
    ///
    /// This is how "bouncy" collisions are.
    #[must_use]
    pub fn with_restitution(mut self, restitution: f64) -> Self {
        self.restitution = restitution;

        self
    }

    /// Spawn into the world.
    #[must_use]
    pub fn spawn<
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const SIZE: usize,
    >(
        self,
        physics: &mut Physics<WIDTH, HEIGHT, STEP, BUCKET, SIZE>,
    ) -> RigidBodyHandle {
        let (inv_mass, inertia) = match self.body_type {
            RigidBodyBuilderType::Dynamic | RigidBodyBuilderType::Kinetic => {
                let mass_properties = self.collider.mass_properties(self.density);
                (
                    mass_properties.mass().recip(),
                    mass_properties.principal_inertia(),
                )
            }
            // Static bodies have infinite mass
            RigidBodyBuilderType::Static => (0.0, 1.0),
        };

        // Spawn the base first
        let pos = Position(self.position);
        let rot = Orientation(self.orientation);
        let inertia = Inertia(inertia);
        let inv_mass = InvMass(inv_mass);
        let friction = Friction(self.friction);
        let restitution = Restitution(self.restitution);
        let collider = Collider(self.collider);
        let entity = physics.world.spawn(BaseRigidBodyBundle {
            pos,
            rot,
            inertia,
            inv_mass,
            friction,
            restitution,
            collider,
        });

        match self.body_type {
            RigidBodyBuilderType::Dynamic => {
                // Insert components needed for linear movement
                let prev_pos = PrevPosition(self.position);
                let trans = Translation(Vec2::zero());
                let vel = Velocity(self.velocity);
                let prev_vel = PrevVelocity(self.velocity);
                physics
                    .world
                    .insert(entity, (prev_pos, trans, vel, prev_vel))
                    .unwrap();

                if self.linear_damping != 1.0 {
                    let lin_damping = LinearDamping(self.linear_damping);
                    physics.world.insert_one(entity, lin_damping).unwrap();
                }

                // Insert components needed for angular movement
                let prev_rot = PrevOrientation(self.orientation);
                let ang_vel = AngularVelocity(self.angular_velocity);
                let prev_ang_vel = PrevAngularVelocity(self.angular_velocity);
                physics
                    .world
                    .insert(entity, (prev_rot, ang_vel, prev_ang_vel))
                    .unwrap();

                if self.angular_damping != 1.0 {
                    let ang_damping = AngularDamping(self.angular_damping);
                    physics.world.insert_one(entity, ang_damping).unwrap();
                }
            }
            RigidBodyBuilderType::Kinetic => todo!(),
            RigidBodyBuilderType::Static => (),
        }

        // Create a handle for the entity
        physics.rigidbodies.wrap_entity(entity)
    }
}

impl Default for RigidBodyBuilder {
    fn default() -> Self {
        let position = Vec2::zero();
        let velocity = Vec2::zero();
        let linear_damping = 1.0;
        let orientation = Rotation::zero();
        let angular_velocity = 0.0;
        let angular_damping = 1.0;
        let density = 1.0;
        let friction = 0.3;
        let restitution = 0.3;
        let collider = Shape::default();
        let body_type = RigidBodyBuilderType::Dynamic;

        Self {
            position,
            velocity,
            linear_damping,
            orientation,
            angular_velocity,
            angular_damping,
            friction,
            density,
            restitution,
            collider,
            body_type,
        }
    }
}

/// Main interface to a rigidbody in the physics engine.
///
/// The rigidbody will be destroyed when this handle and all its clones are dropped.
///
/// All external operations and retrieval of rigidbody information can be done through this.
///
/// Internally it's a reference-counted ECS entity where a weak reference is kept in a separate datastructure which checks if the handle hasn't been dropped yet at the beginning of every physics step.
/// Because the reference is not atomic it can't be dropped while the physics step is running.
#[derive(Debug, Clone)]
pub struct RigidBodyHandle(Rc<Entity>);

impl RigidBodyHandle {
    /// Apply torque as an external angular force.
    pub fn apply_torque<
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const SIZE: usize,
    >(
        &self,
        angular_force: f64,
        physics: &mut Physics<WIDTH, HEIGHT, STEP, BUCKET, SIZE>,
    ) {
        // If no external force is applied before create a new one
        let previous_angular_force = physics
            .rigidbody_opt_value::<&AngularExternalForce>(self)
            .map(|force| force.0)
            .unwrap_or(0.0);

        physics.rigidbody_set_value(self, previous_angular_force + angular_force);
    }

    /// Get the absolute position.
    #[must_use]
    pub fn position<
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const SIZE: usize,
    >(
        &self,
        physics: &Physics<WIDTH, HEIGHT, STEP, BUCKET, SIZE>,
    ) -> Vec2<f64> {
        physics.rigidbody_value::<&Position>(self).0
    }

    /// Get the absolute rotation.
    #[must_use]
    pub fn orientation<
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const SIZE: usize,
    >(
        &self,
        physics: &Physics<WIDTH, HEIGHT, STEP, BUCKET, SIZE>,
    ) -> Rotation {
        physics.rigidbody_value::<&Orientation>(self).0
    }

    /// Get the absolute position combined with orientation.
    #[must_use]
    pub fn iso<
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const SIZE: usize,
    >(
        &self,
        physics: &Physics<WIDTH, HEIGHT, STEP, BUCKET, SIZE>,
    ) -> Iso {
        let pos = physics.rigidbody_value::<&Position>(self).0;
        let rot = physics.rigidbody_value::<&Orientation>(self).0;

        Iso::new(pos, rot)
    }

    /// Get the velocity.
    #[must_use]
    pub fn velocity<
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const SIZE: usize,
    >(
        &self,
        physics: &Physics<WIDTH, HEIGHT, STEP, BUCKET, SIZE>,
    ) -> Vec2<f64> {
        physics.rigidbody_value::<&Velocity>(self).0
    }

    /// Get the angular velocity.
    ///
    /// Assumes the rigidbody is dynamic, otherwise an error is thrown.
    #[must_use]
    pub fn angular_velocity<
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const SIZE: usize,
    >(
        &self,
        physics: &Physics<WIDTH, HEIGHT, STEP, BUCKET, SIZE>,
    ) -> f64 {
        physics.rigidbody_value::<&AngularVelocity>(self).0
    }

    /// Whether the rigidbody is in a sleeping position.
    #[must_use]
    pub fn is_sleeping<
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const SIZE: usize,
    >(
        &self,
        _physics: &Physics<WIDTH, HEIGHT, STEP, BUCKET, SIZE>,
    ) -> bool {
        // TODO
        false
    }

    /// Get the entity reference.
    #[must_use]
    pub(super) fn entity(&self) -> Entity {
        *self.0
    }

    /// Create a weak reference to the rigidbody.
    #[must_use]
    fn downgrade(&self) -> Weak<Entity> {
        Rc::downgrade(&self.0)
    }
}

/// Rigidbody related methods to call on the world of the physics system.
pub struct RigidBodySystems {
    /// List of references to the rigidbody handles so that when a handle is dropped from anywhere we can also destroy the rigidbody.
    rigidbody_references: Vec<(Weak<Entity>, Entity)>,
}

impl RigidBodySystems {
    /// Instantiate with no rigidbodies yet.
    pub fn new() -> Self {
        let rigidbody_references = Vec::new();

        Self {
            rigidbody_references,
        }
    }

    /// Destroy all entities that are dropped.
    ///
    /// Should be called at the beginning of a physics step.
    pub fn destroy_dropped(&mut self, world: &mut World) {
        puffin::profile_scope!("Destroy dropped rigidbodies");

        self.rigidbody_references.retain(|(reference, rigidbody)| {
            if reference.strong_count() == 0 {
                // Remove the rigidbody
                world.despawn(*rigidbody).expect("Entity destroyed twice");

                false
            } else {
                true
            }
        });
    }

    /// Perform an integration step on all rigidbodies where it applies.
    pub fn integrate(&mut self, world: &mut World, dt: f64) {
        puffin::profile_scope!("Integrate");

        // PERF: use premade queries

        let gravity = crate::settings().physics.gravity;

        {
            puffin::profile_scope!("Store position");
            for (_id, (pos, prev_pos)) in world.query_mut::<(&mut Position, &mut PrevPosition)>() {
                prev_pos.0 = pos.0;
            }
        }

        {
            puffin::profile_scope!("Linear damping");
            for (_id, (vel, lin_damping)) in world.query_mut::<(&mut Velocity, &LinearDamping)>() {
                vel.0 *= 1.0 / (1.0 + dt * lin_damping.0);
            }
        }

        {
            puffin::profile_scope!("Gravity");
            for (_id, (vel, inv_mass)) in world.query_mut::<(&mut Velocity, &InvMass)>() {
                vel.0 += (dt * Vec2::new(0.0, gravity)) / inv_mass.0.recip();
            }
        }

        {
            puffin::profile_scope!("External force");
            for (_id, (vel, ext_force, inv_mass)) in
                world.query_mut::<(&mut Velocity, &LinearExternalForce, &InvMass)>()
            {
                vel.0 += (dt * (ext_force.0 + Vec2::new(0.0, gravity))) / inv_mass.0.recip();
            }
        }

        {
            puffin::profile_scope!("Add velocity to translation");
            for (_id, (trans, vel)) in world.query_mut::<(&mut Translation, &Velocity)>() {
                trans.0 += dt * vel.0;
            }
        }

        {
            puffin::profile_scope!("Store orientation");
            for (_id, (rot, prev_rot)) in world.query_mut::<(&Orientation, &mut PrevOrientation)>()
            {
                prev_rot.0 = rot.0;
            }
        }

        {
            puffin::profile_scope!("Angular damping");
            for (_id, (ang_vel, ang_damping)) in
                world.query_mut::<(&mut AngularVelocity, &AngularDamping)>()
            {
                ang_vel.0 *= 1.0 / (1.0 + dt * ang_damping.0);
            }
        }

        {
            puffin::profile_scope!("Add angular velocity to orientation");
            for (_id, (rot, ang_vel)) in world.query_mut::<(&mut Orientation, &AngularVelocity)>() {
                rot.0 += dt * ang_vel.0;
            }
        }
    }

    /// Perform an solve step on all rigidbodies where all velocities are added.
    pub fn update_velocities(&mut self, world: &mut World, dt: f64) {
        // Performance optimization
        let inv_dt = dt.recip();

        {
            puffin::profile_scope!("Store velocity");
            for (_id, (vel, prev_vel)) in world.query_mut::<(&Velocity, &mut PrevVelocity)>() {
                prev_vel.0 = vel.0;
            }
        }

        {
            puffin::profile_scope!("Apply velocity");
            for (_id, (vel, pos, prev_pos, trans)) in
                world.query_mut::<(&mut Velocity, &Position, &PrevPosition, &Translation)>()
            {
                vel.0 = (pos.0 - prev_pos.0 + trans.0) * inv_dt;
            }
        }

        {
            puffin::profile_scope!("Store angular velocity");
            for (_id, (ang_vel, prev_ang_vel)) in
                world.query_mut::<(&AngularVelocity, &mut PrevAngularVelocity)>()
            {
                prev_ang_vel.0 = ang_vel.0;
            }
        }

        {
            puffin::profile_scope!("Apply angular velocity");
            for (_id, (ang_vel, rot, prev_rot)) in
                world.query_mut::<(&mut AngularVelocity, &Orientation, &PrevOrientation)>()
            {
                ang_vel.0 = (rot.0 - prev_rot.0).to_radians() * inv_dt;
            }
        }
    }

    /// Perform an solve step on all rigidbodies where the translation is added to the position.
    pub fn apply_translation(&mut self, world: &mut World) {
        puffin::profile_scope!("Apply translation");
        for (_id, (pos, trans)) in world.query_mut::<(&mut Position, &mut Translation)>() {
            pos.0 += trans.0;
            trans.0 = Vec2::zero();
        }
    }

    /// Wrap a created entity into a handle.
    fn wrap_entity(&mut self, entity: Entity) -> RigidBodyHandle {
        let handle = RigidBodyHandle(Rc::new(entity));

        // Store a weak reference to the handle so we can destroy it when the handle is dropped
        self.rigidbody_references.push((handle.downgrade(), entity));

        handle
    }
}

impl Default for RigidBodySystems {
    fn default() -> Self {
        Self::new()
    }
}

/// Rigidbody entity definition all rigidbodies must at least have.
#[derive(Bundle)]
struct BaseRigidBodyBundle {
    pos: Position,
    rot: Orientation,
    inertia: Inertia,
    inv_mass: InvMass,
    friction: Friction,
    restitution: Restitution,
    collider: Collider,
}

/// Absolute position in the world.
pub(super) struct Position(pub Vec2<f64>);

/// Absolute position in the world for the previous step.
pub(super) struct PrevPosition(pub Vec2<f64>);

/// Linear position offset that will be added in a single step.
pub(super) struct Translation(pub Vec2<f64>);

/// Linear velocity.
pub(super) struct Velocity(pub Vec2<f64>);

/// Linear velocity for the previous step.
pub(super) struct PrevVelocity(pub Vec2<f64>);

/// Linear damping.
pub(super) struct LinearDamping(pub f64);

/// External linear force applied to the whole body evenly.
pub(super) struct LinearExternalForce(pub Vec2<f64>);

/// Absolute orientation, also known as rotation.
pub(super) struct Orientation(pub Rotation);

impl Orientation {
    /// Rotate a relative point to the position in that local space.
    pub fn rotate(&self, point: Vec2<f64>) -> Vec2<f64> {
        self.0.rotate(point)
    }
}

/// Absolute orientation for the previous step.
pub(super) struct PrevOrientation(pub Rotation);

/// Velocity applied to the orientation.
pub(super) struct AngularVelocity(pub f64);

/// Velocity applied to the orientation for the previous step.
pub(super) struct PrevAngularVelocity(pub f64);

/// Angular damping.
pub(super) struct AngularDamping(pub f64);

/// External angular force applied to the whole body evenly.
pub(super) struct AngularExternalForce(pub f64);

/// Inertia tensor, corresponds to mass in rotational terms.
pub(super) struct Inertia(pub f64);

impl Inertia {
    /// Calculate the update in rotation when a position change is applied at a specific point.
    pub fn delta_rotation_at_point(&self, point: Vec2<f64>, impulse: Vec2<f64>) -> f64 {
        // Perpendicular dot product of `point` with `impulse`
        let perp_dot = (point.x * impulse.y) - (point.y * impulse.x);

        self.0.recip() * perp_dot
    }
}

/// Inverse of the mass of a rigidbody.
pub(super) struct InvMass(pub f64);

/// Dynamic and static friction coefficient.
pub(super) struct Friction(pub f64);

impl Friction {
    /// Combine the static frictions between this and another body.
    #[inline]
    pub fn combine_static_frictions(&self, other: &Self) -> f64 {
        (self.static_friction() + other.static_friction()) / 2.0
    }

    /// Combine the dynamic frictions between this and another body.
    #[inline]
    pub fn combine_dynamic_frictions(&self, other: &Self) -> f64 {
        (self.dynamic_friction() + other.dynamic_friction()) / 2.0
    }

    /// Friction that needs to be overcome before resting objects start sliding.
    #[inline]
    pub fn static_friction(&self) -> f64 {
        self.0
    }

    /// Friction that's applied to dynamic moving object after static friction has been overcome.
    #[inline]
    pub fn dynamic_friction(&self) -> f64 {
        self.0
    }
}

/// Restitution coefficient, how bouncy collisions are.
pub(super) struct Restitution(pub f64);

impl Restitution {
    /// Combine the restitutions between this and another body.
    #[inline]
    pub fn combine_restitutions(&self, other: &Self) -> f64 {
        (self.0 + other.0) / 2.0
    }
}

/// Shape for detecting and resolving collisions.
pub(super) struct Collider(pub Shape);

/// Delta position of a point.
pub trait RelativeMotionAtPoint {
    /// Delta position of a point.
    fn relative_motion_at_point(self, point: Vec2<f64>) -> Vec2<f64>;
}

impl RelativeMotionAtPoint for (&Position, &PrevPosition, &Translation, &PrevOrientation) {
    #[inline]
    fn relative_motion_at_point(self, point: Vec2<f64>) -> Vec2<f64> {
        let (pos, prev_pos, trans, prev_rot) = self;

        pos.0 - prev_pos.0 + trans.0 + point - prev_rot.0.rotate(point)
    }
}

/// Calculate the world position of a relative point on this body without rotation in mind.
pub trait LocalToWorld {
    /// Calculate the world position of a relative point on this body without rotation in mind.
    fn local_to_world(self, point: Vec2<f64>) -> Vec2<f64>;
}

impl LocalToWorld for (&Position, &Translation, &Orientation) {
    #[inline]
    fn local_to_world(self, point: Vec2<f64>) -> Vec2<f64> {
        let (pos, trans, rot) = self;

        // Then translate it to the position
        pos.0 + trans.0 + rot.rotate(point)
    }
}

/// Apply a positional impulse at a point.
pub trait ApplyPositionalImpulse {
    /// Apply a positional impulse at a point.
    ///
    // TODO: can we remove the sign by directly negating the impulse?
    fn apply_positional_impulse(self, positional_impulse: Vec2<f64>, point: Vec2<f64>, sign: f64);
}

impl ApplyPositionalImpulse for (&mut Translation, &mut Orientation, &InvMass, &Inertia) {
    fn apply_positional_impulse(self, positional_impulse: Vec2<f64>, point: Vec2<f64>, sign: f64) {
        let (trans, rot, inv_mass, inertia) = self;

        trans.0 += sign * positional_impulse * inv_mass.0;
        rot.0 += sign * inertia.delta_rotation_at_point(point, positional_impulse);
    }
}

/// Calculate the contact velocity based on a local relative rotated point.
pub trait ContactVelocity {
    /// Calculate the contact velocity based on a local relative rotated point.
    fn contact_velocity(self, point: Vec2<f64>) -> Vec2<f64>;
}

impl ContactVelocity for (&Velocity, &AngularVelocity) {
    #[inline]
    fn contact_velocity(self, point: Vec2<f64>) -> Vec2<f64> {
        let (vel, ang_vel) = self;

        // Perpendicular
        let perp = Vec2::new(-point.y, point.x);

        vel.0 + ang_vel.0 * perp
    }
}

impl ContactVelocity for (&PrevVelocity, &PrevAngularVelocity) {
    #[inline]
    fn contact_velocity(self, point: Vec2<f64>) -> Vec2<f64> {
        let (prev_vel, prev_ang_vel) = self;

        // Perpendicular
        let perp = Vec2::new(-point.y, point.x);

        prev_vel.0 + prev_ang_vel.0 * perp
    }
}

/// Calculate generalized inverse mass at a relative point along the normal vector.
pub trait InverseMassAtRelativePoint {
    /// Calculate generalized inverse mass at a relative point along the normal vector.
    fn inverse_mass_at_relative_point(self, point: Vec2<f64>, normal: Vec2<f64>) -> f64;
}

impl InverseMassAtRelativePoint for (&InvMass, &Inertia) {
    #[inline]
    fn inverse_mass_at_relative_point(self, point: Vec2<f64>, normal: Vec2<f64>) -> f64 {
        let (inv_mass, inertia) = self;

        // Perpendicular dot product of `point` with `normal`
        let perp_dot = (point.x * normal.y) - (point.y * normal.x);

        inv_mass.0 + inertia.0.recip() * perp_dot.powi(2)
    }
}

/// Apply a velocity change at a point.
pub trait ApplyVelocityImpulse {
    /// Apply a velocity change at a point.
    fn apply_velocity_impulse(self, velocity_impulse: Vec2<f64>, point: Vec2<f64>, sign: f64);
}

impl ApplyVelocityImpulse for (&mut Velocity, &mut AngularVelocity, &InvMass, &Inertia) {
    #[inline]
    fn apply_velocity_impulse(self, velocity_impulse: Vec2<f64>, point: Vec2<f64>, sign: f64) {
        let (vel, ang_vel, inv_mass, inertia) = self;

        vel.0 += sign * velocity_impulse * inv_mass.0;
        ang_vel.0 += sign * inertia.delta_rotation_at_point(point, velocity_impulse);
    }
}
