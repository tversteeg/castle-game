use std::rc::{Rc, Weak};

use hecs::{Bundle, ComponentRef, Entity, Query, View, World};
use vek::{Aabr, Vec2};

use crate::math::{Iso, Rotation};

use super::{
    collision::{shape::Shape, CollisionResponse, CollisionState},
    Physics,
};

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

/// Rigidbody builder types.
enum RigidBodyBuilderType {
    Dynamic,
    Kinetic,
    Static,
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

        physics.rigidbody_set_value(
            self,
            AngularExternalForce(previous_angular_force + angular_force),
        );
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

    /// Get the bounding box.
    #[must_use]
    pub fn aabr<
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const SIZE: usize,
    >(
        &self,
        physics: &Physics<WIDTH, HEIGHT, STEP, BUCKET, SIZE>,
    ) -> Aabr<f64> {
        physics
            .rigidbody_value::<&Collider>(self)
            .0
            .aabr(self.iso(physics))
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
                vel.0 += (dt * ext_force.0) / inv_mass.0.recip();
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
            puffin::profile_scope!("Angular external forces");
            for (_id, (ang_vel, ang_ext_force, inertia)) in
                world.query_mut::<(&mut AngularVelocity, &AngularExternalForce, &Inertia)>()
            {
                ang_vel.0 += dt * inertia.0.recip() * ang_ext_force.0;
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

/// Simplified query for a rigidbody.
///
/// Everything that's not optional here must also be part of the [`BaseRigidBodyBundle`].
#[derive(Debug, Query)]
pub struct RigidBodyQuery<'a> {
    pub pos: &'a mut Position,
    pub inertia: &'a Inertia,
    pub inv_mass: &'a InvMass,
    pub friction: &'a Friction,
    pub rot: &'a mut Orientation,
    pub restitution: &'a Restitution,
    prev_pos: Option<&'a mut PrevPosition>,
    trans: Option<&'a mut Translation>,
    vel: Option<&'a mut Velocity>,
    prev_vel: Option<&'a mut PrevVelocity>,
    prev_rot: Option<&'a mut PrevOrientation>,
    ang_vel: Option<&'a mut AngularVelocity>,
    prev_ang_vel: Option<&'a mut PrevAngularVelocity>,
}

impl<'a> RigidBodyQuery<'a> {
    /// Apply a positional impulse at a point.
    ///
    // TODO: can we remove the sign by directly negating the impulse?
    pub fn apply_positional_impulse(
        &mut self,
        positional_impulse: Vec2<f64>,
        point: Vec2<f64>,
        sign: f64,
    ) {
        if let Some(trans) = self.trans.as_mut() {
            trans.0 += sign * positional_impulse * self.inv_mass.0;
            self.rot.0 += sign * self.delta_rotation_at_point(point, positional_impulse);
        }
    }

    /// Apply a velocity change at a point.
    #[inline]
    pub fn apply_velocity_impulse(
        &mut self,
        velocity_impulse: Vec2<f64>,
        point: Vec2<f64>,
        sign: f64,
    ) {
        if let Some(vel) = self.vel.as_mut() {
            vel.0 += sign * velocity_impulse * self.inv_mass.0;
        }

        let delta_rotation = self.delta_rotation_at_point(point, velocity_impulse);

        if let Some(ang_vel) = self.ang_vel.as_mut() {
            ang_vel.0 += sign * delta_rotation;
        }
    }

    /// Rotate a relative point to the position in that local space.
    pub fn rotate(&self, point: Vec2<f64>) -> Vec2<f64> {
        self.rot.0.rotate(point)
    }

    /// Calculate the update in rotation when a position change is applied at a specific point.
    pub fn delta_rotation_at_point(&self, point: Vec2<f64>, impulse: Vec2<f64>) -> f64 {
        // Perpendicular dot product of `point` with `impulse`
        let perp_dot = (point.x * impulse.y) - (point.y * impulse.x);

        self.inertia.0.recip() * perp_dot
    }

    /// Delta position of a point.
    #[inline]
    pub fn relative_motion_at_point(&self, point: Vec2<f64>) -> Vec2<f64> {
        self.pos.0 - self.previous_position() + self.translation() + point
            - self.previous_orientation().rotate(point)
    }

    /// Calculate generalized inverse mass at a relative point along the normal vector.
    #[inline]
    pub fn inverse_mass_at_relative_point(&self, point: Vec2<f64>, normal: Vec2<f64>) -> f64 {
        self.inv_mass
            .inverse_mass_at_relative_point(self.inertia, point, normal)
    }

    /// Calculate the contact velocity based on a local relative rotated point.
    #[inline]
    pub fn contact_velocity(&self, point: Vec2<f64>) -> Option<Vec2<f64>> {
        if let Some((vel, ang_vel)) = self.vel.as_ref().zip(self.ang_vel.as_ref()) {
            // Perpendicular
            let perp = Vec2::new(-point.y, point.x);

            Some(vel.0 + ang_vel.0 * perp)
        } else {
            None
        }
    }

    /// Calculate the contact velocity based on a local relative rotated point.
    #[inline]
    pub fn prev_contact_velocity(&self, point: Vec2<f64>) -> Option<Vec2<f64>> {
        if let Some((prev_vel, prev_ang_vel)) =
            self.prev_vel.as_ref().zip(self.prev_ang_vel.as_ref())
        {
            // Perpendicular
            let perp = Vec2::new(-point.y, point.x);

            Some(prev_vel.0 + prev_ang_vel.0 * perp)
        } else {
            None
        }
    }

    /// Calculate the world position of a relative point on this body without rotation in mind.
    #[inline]
    pub fn local_to_world(&self, point: Vec2<f64>) -> Vec2<f64> {
        // Then translate it to the position
        self.pos.0 + self.translation() + self.rotate(point)
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

    /// Friction that needs to be overcome before resting objects start sliding.
    #[inline]
    pub fn static_friction(&self) -> f64 {
        self.friction.0
    }

    /// Friction that's applied to dynamic moving object after static friction has been overcome.
    #[inline]
    pub fn dynamic_friction(&self) -> f64 {
        self.friction.0
    }

    /// Combine the restitutions between this and another body.
    #[inline]
    pub fn combine_restitutions(&self, other: &Self) -> f64 {
        (self.restitution.0 + other.restitution.0) / 2.0
    }

    /// Whether the body cannot move and has infinite mass.
    #[inline]
    pub fn is_static(&self) -> bool {
        self.vel.is_none() && self.inv_mass.0 == 0.0
    }

    /// The translation or zero if it's static.
    #[inline]
    pub fn translation(&self) -> Vec2<f64> {
        if let Some(trans) = self.trans.as_ref() {
            trans.0
        } else {
            Vec2::zero()
        }
    }

    /// The previous position or the current position if it's static.
    #[inline]
    pub fn previous_position(&self) -> Vec2<f64> {
        if let Some(prev_pos) = self.prev_pos.as_ref() {
            prev_pos.0
        } else {
            self.pos.0
        }
    }

    /// The previous orientation or the current orientation if it's static.
    #[inline]
    pub fn previous_orientation(&self) -> Rotation {
        if let Some(prev_rot) = self.prev_rot.as_ref() {
            prev_rot.0
        } else {
            self.rot.0
        }
    }
}

/// Absolute position in the world.
#[derive(Debug, Default)]
pub struct Position(pub Vec2<f64>);

/// Absolute position in the world for the previous step.
#[derive(Debug)]
pub struct PrevPosition(pub Vec2<f64>);

/// Linear position offset that will be added in a single step.
#[derive(Debug, Default)]
pub struct Translation(pub Vec2<f64>);

/// Linear velocity.
#[derive(Debug, Default)]
pub struct Velocity(pub Vec2<f64>);

/// Linear velocity for the previous step.
#[derive(Debug)]
pub struct PrevVelocity(pub Vec2<f64>);

/// Linear damping.
#[derive(Debug)]
pub struct LinearDamping(pub f64);

impl Default for LinearDamping {
    fn default() -> Self {
        Self(1.0)
    }
}

/// External linear force applied to the whole body evenly.
#[derive(Debug, Default)]
pub struct LinearExternalForce(pub Vec2<f64>);

/// Absolute orientation, also known as rotation.
#[derive(Debug, Default)]
pub struct Orientation(pub Rotation);

/// Absolute orientation for the previous step.
#[derive(Debug)]
pub struct PrevOrientation(pub Rotation);

/// Velocity applied to the orientation.
#[derive(Debug, Default)]
pub struct AngularVelocity(pub f64);

/// Velocity applied to the orientation for the previous step.
#[derive(Debug)]
pub struct PrevAngularVelocity(pub f64);

/// Angular damping.
#[derive(Debug)]
pub struct AngularDamping(pub f64);

impl Default for AngularDamping {
    fn default() -> Self {
        Self(1.0)
    }
}

/// External angular force applied to the whole body evenly.
#[derive(Debug, Default)]
pub struct AngularExternalForce(pub f64);

/// Inertia tensor, corresponds to mass in rotational terms.
#[derive(Debug, Default, Clone)]
pub struct Inertia(pub f64);

/// Inverse of the mass of a rigidbody.
#[derive(Debug, Default, Clone)]
pub struct InvMass(pub f64);

impl InvMass {
    /// Calculate generalized inverse mass at a relative point along the normal vector.
    #[inline]
    pub fn inverse_mass_at_relative_point(
        &self,
        inertia: &Inertia,
        point: Vec2<f64>,
        normal: Vec2<f64>,
    ) -> f64 {
        // When the body is static the inverse mass is always zero, independent of the point
        if self.0 == 0.0 {
            return 0.0;
        }

        // Perpendicular dot product of `point` with `normal`
        let perp_dot = (point.x * normal.y) - (point.y * normal.x);

        self.0 + inertia.0.recip() * perp_dot.powi(2)
    }
}

/// Dynamic and static friction coefficient.
#[derive(Debug, Default)]
pub struct Friction(pub f64);

/// Restitution coefficient, how bouncy collisions are.
#[derive(Debug, Default)]
pub struct Restitution(pub f64);

/// Shape for detecting and resolving collisions.
#[derive(Debug, Default)]
pub(super) struct Collider(pub Shape);
