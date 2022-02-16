use bevy::{
    math::{Vec2, Vec3},
    prelude::{Assets, Bundle, Color, Component, Mesh, Transform},
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::{ColorMaterial, MaterialMesh2dBundle},
    utils::tracing,
};
use bevy_inspector_egui::Inspectable;
use geo::{prelude::IsConvex, Polygon};
use heron::{
    rapier_plugin::rapier2d::{math::Point, prelude::ColliderBuilder},
    CollisionShape, CustomCollisionShape, Velocity,
};

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
pub trait ToCollisionShape {
    /// Convert the polygon to a collision shape by taking the outline.
    fn to_collision_shape(&self) -> CollisionShape;
}

impl ToCollisionShape for Polygon<f32> {
    #[tracing::instrument(name = "converting polygon to collision shape")]
    fn to_collision_shape(&self) -> CollisionShape {
        // If the polygon is convex just create a convex hull for it, which is better performance-wise
        if self.exterior().is_convex() || true {
            // Convert the polygon points to coordinates
            let points = self
                .exterior()
                .points_iter()
                .map(|point| Vec3::new(point.x(), point.y(), 0.0))
                .collect::<Vec<_>>();

            CollisionShape::ConvexHull {
                points,
                border_radius: None,
            }
        } else {
            // Convert the polygon points to coordinates
            let points = self
                .exterior()
                .points_iter()
                .map(|point| Point::new(point.x(), point.y()))
                .collect::<Vec<_>>();

            // Create a custom collider for a concave hull
            let shape = CustomCollisionShape::new(ColliderBuilder::polyline(points, None));

            CollisionShape::Custom { shape }
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
            .points_iter()
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
    collision_shape: CollisionShape,
    /// The velocity.
    #[inspectable(ignore)]
    velocity: Velocity,
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
        let collision_shape = polygon.to_collision_shape();

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

        let velocity = Velocity::default();

        Self {
            shape,
            material_mesh,
            collision_shape,
            velocity,
        }
    }
}
