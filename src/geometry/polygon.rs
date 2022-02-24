use bevy::{
    math::Vec2,
    prelude::{Assets, Bundle, Color, Component, Mesh, Transform},
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::{ColorMaterial, MaterialMesh2dBundle},
    utils::tracing,
};
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::prelude::{ColliderShape, ColliderShapeComponent, RigidBodyVelocityComponent};
use geo::{prelude::IsConvex, Polygon};

/// Convert a geo polygon to a mesh.
pub trait ToMesh {
    /// Convert the polygon to a mesh by applying the earcut algorithm.
    fn to_mesh(&self) -> Mesh;
}

impl ToMesh for Polygon<f32> {
    #[tracing::instrument(name = "converting polygon to mesh")]
    fn to_mesh(&self) -> Mesh {
        // Convert the polygon to triangles
        let (vertices, indices) = self.triangulate();

        // Create a new mesh.
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        // Set the indices
        mesh.set_indices(Some(Indices::U32(
            indices.into_iter().map(|i| i as u32).collect(),
        )));

        // Set the vertices
        mesh.set_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vertices
                .chunks(2)
                .map(|xy| [xy[0] as f32, xy[1] as f32, 0.0])
                .collect::<Vec<_>>(),
        );

        // Set the normals
        let mut normals = Vec::new();
        normals.resize(vertices.len() / 2, [0.0, 0.0, 1.0]);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

        // Set the UVs
        let mut uvs = Vec::new();
        uvs.resize(vertices.len() / 2, [0.0, 0.0]);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        mesh
    }
}

/// Convert a geo polygon to a collision shape.
pub trait ToColliderShape {
    /// Convert the polygon to a collision shape by taking the outline.
    fn to_collider_shape(&self) -> ColliderShape;
}

impl ToColliderShape for Polygon<f32> {
    #[tracing::instrument(name = "converting polygon to collision shape")]
    fn to_collider_shape(&self) -> ColliderShape {
        // Convert the polygon points to coordinates
        let points = self
            .exterior()
            .points()
            .map(|point| nalgebra::point![point.x(), point.y()])
            .collect::<Vec<_>>();

        // If the polygon is convex just create a convex hull for it, which is better performance-wise
        if self.exterior().is_convex() {
            // TODO: handle error
            ColliderShape::convex_hull(&points).unwrap()
        } else {
            // TODO: fix
            // ColliderShape::polyline(points, None)
            ColliderShape::convex_hull(&points).unwrap()
        }
    }
}

/// Triangulate a polygon.
trait Triangulate {
    /// Triangulate using earcutr.
    ///
    /// Returns vertices and indices.
    fn triangulate(&self) -> (Vec<f32>, Vec<usize>);
}

impl Triangulate for Polygon<f32> {
    #[tracing::instrument(name = "triangulating polygon")]
    fn triangulate(&self) -> (Vec<f32>, Vec<usize>) {
        // Convert the polygon points to coordinates
        let coordinates = self
            .exterior()
            .points()
            .map(|point| vec![point.x(), point.y()])
            .collect::<Vec<Vec<_>>>();

        // Convert the coordinates to indices and holes
        let (vertices, hole_indices, dimensions) = earcutr::flatten(&vec![coordinates]);

        // Triangulate the polygon
        let indices = earcutr::earcut(&vertices, &hole_indices, dimensions);

        (vertices, indices)
    }
}

/// Mark the entity as a polygon.
#[derive(Debug, Component, Inspectable)]
pub struct PolygonComponent;

/// The polygon component.
#[derive(Bundle, Inspectable)]
pub struct PolygonBundle {
    /// Marking it as a polygon.
    shape: PolygonComponent,
    /// The mesh and the material.
    #[bundle]
    #[inspectable(ignore)]
    material_mesh: MaterialMesh2dBundle<ColorMaterial>,
    /// The collision shape.
    #[inspectable(ignore)]
    collider_shape: ColliderShapeComponent,
    /// The velocity.
    #[inspectable(ignore)]
    velocity: RigidBodyVelocityComponent,
}

impl PolygonBundle {
    /// Construct a new polygon with a single color and position.
    pub fn new(
        polygon: &Polygon<f32>,
        color: Color,
        position: Vec2,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Self {
        // Create the collision shape
        let collider_shape = ColliderShapeComponent(polygon.to_collider_shape());

        let material_mesh = MaterialMesh2dBundle {
            // Create the mesh and add it to the global list of meshes
            mesh: meshes.add(polygon.to_mesh()).into(),
            // Set the position
            transform: Transform::from_xyz(position.x, position.y, 0.0),
            // Create the material from a single color and add it to the global list of materials
            material: materials.add(ColorMaterial::from(color)),
            ..Default::default()
        };

        let shape = PolygonComponent;

        let velocity = RigidBodyVelocityComponent::default();

        Self {
            shape,
            material_mesh,
            collider_shape,
            velocity,
        }
    }
}
