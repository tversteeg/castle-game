use bevy::{
    math::Vec2,
    prelude::{Assets, Bundle, Color, Component, Mesh, Transform},
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::{ColorMaterial, MaterialMesh2dBundle},
    utils::tracing,
};
use bevy_inspector_egui::{
    egui::{
        plot::{Legend, Line, Plot, Value, Values},
        Grid, Ui,
    },
    Context, Inspectable,
};
use bevy_rapier2d::prelude::ColliderShape;
use geo::{prelude::IsConvex, LineString, Polygon as GeoPolygon};
use std::ops::{Deref, DerefMut};

/// Convert a geo polygon to a mesh.
pub trait ToMesh {
    /// Convert the polygon to a mesh by applying the earcut algorithm.
    fn to_mesh(&self) -> Mesh;
}

/// Convert a geo polygon to a collision shape.
pub trait ToColliderShape {
    /// Convert the polygon to a collision shape by taking the outline.
    fn to_collider_shape(&self) -> ColliderShape;
}

/// Triangulate a polygon.
trait Triangulate {
    /// Triangulate using earcutr.
    ///
    /// Returns vertices and indices.
    fn triangulate(&self) -> (Vec<f32>, Vec<usize>);
}

/// Mark the entity as a polygon.
#[derive(Debug, Clone, Component)]
pub struct Polygon(GeoPolygon<f32>);

impl Polygon {
    /// Construct a new polygon.
    pub fn new(exterior: LineString<f32>, interiors: Vec<LineString<f32>>) -> Self {
        Self(GeoPolygon::new(exterior, interiors))
    }
}

impl From<GeoPolygon<f32>> for Polygon {
    fn from(polygon: GeoPolygon<f32>) -> Self {
        Self(polygon)
    }
}

impl Deref for Polygon {
    type Target = GeoPolygon<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Polygon {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ToMesh for Polygon {
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

impl ToColliderShape for Polygon {
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

impl Triangulate for Polygon {
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

impl Inspectable for Polygon {
    type Attributes = ();

    fn ui(&mut self, ui: &mut Ui, _: Self::Attributes, context: &mut Context) -> bool {
        Grid::new(context.id()).show(ui, |ui| {
            // Plot the polygon
            ui.label("Plot");
            let plot = Plot::new("polygon")
                .legend(Legend::default())
                .data_aspect(0.8)
                .min_size(bevy_inspector_egui::egui::Vec2::new(250.0, 250.0))
                .show_x(true)
                .show_y(true);
            plot.show(ui, |plot_ui| {
                // TODO: plot interior
                plot_ui.line(
                    Line::new(Values::from_values_iter(
                        self.exterior()
                            .coords()
                            .map(|coord| Value::new(coord.x, coord.y)),
                    ))
                    .name("exterior"),
                )
            });
        });

        false
    }
}

/// The polygon component filled with a single color.
#[derive(Bundle, Inspectable)]
pub struct PolygonShapeBundle {
    /// The polygon.
    polygon: Polygon,
    /// The mesh and the material.
    #[bundle]
    #[inspectable(ignore)]
    material_mesh: MaterialMesh2dBundle<ColorMaterial>,
}

impl PolygonShapeBundle {
    /// Construct a new polygon with a single color and position.
    pub fn new(
        polygon: Polygon,
        color: Color,
        position: Vec2,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Self {
        let material_mesh = MaterialMesh2dBundle {
            // Create the mesh and add it to the global list of meshes
            mesh: meshes.add(polygon.to_mesh()).into(),
            // Set the position
            transform: Transform::from_xyz(position.x, position.y, 0.0),
            // Create the material from a single color and add it to the global list of materials
            material: materials.add(ColorMaterial::from(color)),
            ..Default::default()
        };

        Self {
            material_mesh,
            polygon,
        }
    }
}
