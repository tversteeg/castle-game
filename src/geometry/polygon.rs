use bevy::{
    math::Vec2,
    prelude::{Assets, Bundle, Color, Component, Mesh},
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
use lyon_path::{geom::euclid::Point2D, math::Transform, Path};
use lyon_tessellation::{LineCap, StrokeOptions};
use std::ops::{Deref, DerefMut};

use crate::draw::{
    colored_mesh::ColoredMeshBundle,
    mesh::{MeshBuffers, ToMesh},
};

/// Lyon tolerance for generating a mesh from the stroke.
pub const STROKE_TOLERANCE: f32 = 0.1;

/// Convert a geo polygon to a collision shape.
pub trait ToColliderShape {
    /// Convert the polygon to a collision shape by taking the outline.
    fn to_collider_shape(&self) -> ColliderShape;
}

/// Mark the entity as a polygon.
#[derive(Debug, Clone, Component)]
pub struct Polygon {
    /// The color to render the fill.
    pub fill: Option<Color>,
    /// The color to render the stroke and the size.
    pub stroke: Option<(Color, f32)>,
    /// The polygon geometry.
    polygon: GeoPolygon<f32>,
}

impl Polygon {
    /// Construct a new polygon.
    pub fn new(exterior: LineString<f32>, interiors: Vec<LineString<f32>>) -> Self {
        Self {
            polygon: GeoPolygon::new(exterior, interiors),
            fill: None,
            stroke: None,
        }
    }

    /// Triangulate the polygon.
    ///
    /// Uses earcutr instead of lyon, generates better collider meshes.
    pub fn triangulate(&self) -> (Vec<f32>, Vec<usize>) {
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

impl From<GeoPolygon<f32>> for Polygon {
    fn from(polygon: GeoPolygon<f32>) -> Self {
        Self {
            polygon,
            fill: None,
            stroke: None,
        }
    }
}

impl From<&Polygon> for Path {
    /// Build a lyon path from the polygon.
    fn from(polygon: &Polygon) -> Self {
        let mut builder = Path::builder();

        let mut exterior = polygon.exterior().points();

        // Get the first to begin
        let first = exterior.next().expect("Polygon is empty");
        builder.begin(Point2D::new(first.x(), first.y()));

        for point in exterior {
            builder.line_to(Point2D::new(point.x(), point.y()));
        }

        builder.end(true);

        builder.build()
    }
}

impl Deref for Polygon {
    type Target = GeoPolygon<f32>;

    fn deref(&self) -> &Self::Target {
        &self.polygon
    }
}

impl DerefMut for Polygon {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.polygon
    }
}

impl ToMesh for Polygon {
    fn buffers(&self) -> (Vec<[f32; 3]>, Vec<u32>, Vec<[f32; 4]>) {
        let mut buffers = MeshBuffers::new();

        // Add the fill first, so the stroke will be placed on top over it
        if let Some(fill_color) = self.fill {
            buffers.append_fill(
                Path::from(self).into_iter(),
                Transform::default(),
                fill_color,
            );
        }

        // Add the stroke
        if let Some((stroke_color, stroke_size)) = self.stroke {
            let stroke_options = StrokeOptions::tolerance(STROKE_TOLERANCE)
                .with_line_width(stroke_size)
                .with_line_cap(LineCap::Square);

            buffers.append_stroke(
                Path::from(self).into_iter(),
                &stroke_options,
                Transform::default(),
                stroke_color,
            );
        }

        buffers.buffers()
    }
}

impl ToColliderShape for Polygon {
    #[tracing::instrument(name = "converting polygon to collision shape")]
    fn to_collider_shape(&self) -> ColliderShape {
        // If the polygon is convex just create a convex hull for it, which is better performance-wise
        if self.exterior().is_convex() {
            // Convert the polygon points to coordinates
            let points = self
                .exterior()
                .points()
                .map(|point| nalgebra::point![point.x(), point.y()])
                .collect::<Vec<_>>();

            // TODO: handle error
            ColliderShape::convex_hull(&points).unwrap()
        } else {
            // Triangulate the polygon first with a simple mesh without a stroke
            let (vertices, indices) = self.triangulate();

            // Convert the vertices to rapier vertices
            assert!(vertices.len() % 2 == 0);
            let vertices = vertices
                .chunks_exact(2)
                .map(|xy| nalgebra::point![xy[0], xy[1]])
                .collect::<Vec<_>>();

            // Convert the indices to rapier indices
            assert!(indices.len() % 3 == 0);
            let indices = indices
                .chunks_exact(3)
                .map(|indices| [indices[0] as u32, indices[1] as u32, indices[2] as u32])
                .collect::<Vec<_>>();

            ColliderShape::trimesh(vertices, indices)
        }
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
    mesh: ColoredMeshBundle,
}

impl PolygonShapeBundle {
    /// Construct a new polygon with a single color and position.
    pub fn new(
        mut polygon: Polygon,
        fill: Option<Color>,
        stroke: Option<(Color, f32)>,
        position: Vec2,
        meshes: &mut Assets<Mesh>,
    ) -> Self {
        polygon.fill = fill;
        polygon.stroke = stroke;

        let mesh = ColoredMeshBundle::new(position, meshes.add(polygon.to_mesh()));

        Self { mesh, polygon }
    }
}
