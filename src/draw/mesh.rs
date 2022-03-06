use std::slice::Iter;

use bevy::{
    prelude::{Color, Mesh},
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    utils::tracing,
};
use lyon_tessellation::{
    geom::euclid::default::Transform2D, math::Point, path::PathEvent, BuffersBuilder, FillOptions,
    FillTessellator, FillVertex, FillVertexConstructor, StrokeOptions, StrokeTessellator,
    StrokeVertex, StrokeVertexConstructor, VertexBuffers,
};
use usvg::{NodeKind, Paint, Path, PathSegment, Transform, Tree};

/// A custom vertex constructor for lyon, creates bevy vertices.
struct BevyVertexConstructor {
    /// The transform to apply to all vertices.
    transform: Transform,
}

impl FillVertexConstructor<[f32; 3]> for BevyVertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> [f32; 3] {
        let pos = vertex.position();

        // Transform the 2D point
        // TODO: remove ugly casts
        let (x, y) = self.transform.apply(pos.x as f64, pos.y as f64);

        [x as f32, y as f32, 0.0]
    }
}

impl StrokeVertexConstructor<[f32; 3]> for BevyVertexConstructor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> [f32; 3] {
        let pos = vertex.position();

        // Transform the 2D point
        // TODO: remove ugly casts
        let (x, y) = self.transform.apply(pos.x as f64, pos.y as f64);

        [x as f32, y as f32, 0.0]
    }
}

/// Convert a SVG to a mesh.
pub fn svg_to_mesh(svg: &Tree) -> Mesh {
    bevy::log::trace!("Converting SVG paths to mesh");

    // The resulting vertex and index buffers
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut colors = Vec::new();

    for node in svg.root().descendants() {
        if let NodeKind::Path(ref path) = *node.borrow() {
            bevy::log::trace!("Parsing SVG path node");

            // Convert the fill to a polygon
            if let Some(ref fill) = path.fill {
                let mut buffers = svg_path_fill_to_vertex_buffers(&path, vertices.len() as u32);

                // Merge the buffers
                vertices.append(&mut buffers.vertices);
                indices.append(&mut buffers.indices);
                // Fill the buffer with the same size as the vertices with colors
                colors.resize(
                    vertices.len(),
                    svg_color_to_bevy(&fill.paint, fill.opacity.to_u8()),
                );
            }

            // Convert the stroke to a polygon
            if let Some(ref stroke) = path.stroke {
                let mut buffers = svg_path_stroke_to_vertex_buffers(&path, vertices.len() as u32);

                // Merge the buffers
                vertices.append(&mut buffers.vertices);
                indices.append(&mut buffers.indices);
                // Fill the buffer with the same size as the vertices with colors
                colors.resize(
                    vertices.len(),
                    svg_color_to_bevy(&stroke.paint, stroke.opacity.to_u8()),
                );
            }
        }
    }

    convert_buffers_into_mesh(vertices, indices, colors)
}

/// Convert a SVG path fill to a mesh.
#[tracing::instrument(name = "converting SVG path fill to vertex buffers")]
fn svg_path_fill_to_vertex_buffers(
    path: &Path,
    indices_offset: u32,
) -> VertexBuffers<[f32; 3], u32> {
    bevy::log::trace!("Converting SVG path fill to vertex buffers");

    // The resulting vertex and index buffers
    let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();

    // Use our custom vertex constructor to create a bevy vertex buffer
    let mut vertex_builder = BuffersBuilder::new(
        &mut buffers,
        BevyVertexConstructor {
            transform: path.transform,
        },
    );

    // Tesselate the fill
    let mut tessellator = FillTessellator::new();
    let result = tessellator.tessellate(
        PathConvIter::from_svg_path(&path),
        &FillOptions::default(),
        &mut vertex_builder,
    );
    assert!(result.is_ok());

    // Add the offset so multiple items can be merged
    if indices_offset != 0 {
        buffers
            .indices
            .iter_mut()
            .for_each(|index| *index += indices_offset);
    }

    buffers
}

/// Convert a SVG path stroke to a mesh.
#[tracing::instrument(name = "converting SVG path stroke to vertex buffers")]
fn svg_path_stroke_to_vertex_buffers(
    path: &Path,
    indices_offset: u32,
) -> VertexBuffers<[f32; 3], u32> {
    bevy::log::trace!("Converting SVG path stroke to vertex buffers");

    // The resulting vertex and index buffers
    let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();

    // Use our custom vertex constructor to create a bevy vertex buffer
    let mut vertex_builder = BuffersBuilder::new(
        &mut buffers,
        BevyVertexConstructor {
            transform: path.transform,
        },
    );

    // Tesselate the fill
    let mut tessellator = StrokeTessellator::new();
    let result = tessellator.tessellate(
        PathConvIter::from_svg_path(&path),
        &StrokeOptions::default(),
        &mut vertex_builder,
    );
    assert!(result.is_ok());

    // Add the offset so multiple items can be merged
    if indices_offset != 0 {
        buffers
            .indices
            .iter_mut()
            .for_each(|index| *index += indices_offset);
    }

    buffers
}

/// Convert the vertex buffers to a mesh.
#[tracing::instrument(name = "converting vertex and index buffers into mesh")]
fn convert_buffers_into_mesh(
    vertices: Vec<[f32; 3]>,
    indices: Vec<u32>,
    colors: Vec<[f32; 4]>,
) -> Mesh {
    bevy::log::trace!("Creating mesh");

    // Create the mesh
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    // Set the indices
    mesh.set_indices(Some(Indices::U32(indices)));

    // Set the vertices
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    // Set the colors
    //mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, colors);

    mesh
}

// Taken from https://github.com/nical/lyon/blob/74e6b137fea70d71d3b537babae22c6652f8843e/examples/wgpu_svg/src/main.rs
struct PathConvIter<'a> {
    iter: Iter<'a, PathSegment>,
    prev: Point,
    first: Point,
    needs_end: bool,
    deferred: Option<PathEvent>,
    scale: Transform2D<f32>,
}

impl<'a> PathConvIter<'a> {
    /// Convert a SVG path to the iterator for the tessellator.
    pub fn from_svg_path(path: &'a Path) -> Self {
        Self {
            iter: path.data.iter(),
            first: Point::zero(),
            prev: Point::zero(),
            deferred: None,
            needs_end: false,
            // For some reason the local transform of some paths has negative scale values
            // Here we correct to positive values
            scale: Transform2D::scale(
                if path.transform.a < 0.0 { -1.0 } else { 1.0 },
                if path.transform.d < 0.0 { -1.0 } else { 1.0 },
            ),
        }
    }
}

impl<'l> Iterator for PathConvIter<'l> {
    type Item = PathEvent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.deferred.is_some() {
            return self.deferred.take();
        }
        let mut return_event = None;
        let next = self.iter.next();
        match next {
            Some(PathSegment::MoveTo { x, y }) => {
                if self.needs_end {
                    let last = self.prev;
                    let first = self.first;
                    self.needs_end = false;
                    self.prev = Point::new(*x as f32, *y as f32);
                    self.deferred = Some(PathEvent::Begin { at: self.prev });
                    self.first = self.prev;
                    return_event = Some(PathEvent::End {
                        last,
                        first,
                        close: false,
                    });
                } else {
                    self.first = Point::new(*x as f32, *y as f32);
                    return_event = Some(PathEvent::Begin { at: self.first });
                }
            }
            Some(PathSegment::LineTo { x, y }) => {
                self.needs_end = true;
                let from = self.prev;
                self.prev = Point::new(*x as f32, *y as f32);
                return_event = Some(PathEvent::Line {
                    from,
                    to: self.prev,
                });
            }
            Some(PathSegment::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            }) => {
                self.needs_end = true;
                let from = self.prev;
                self.prev = Point::new(*x as f32, *y as f32);
                return_event = Some(PathEvent::Cubic {
                    from,
                    ctrl1: Point::new(*x1 as f32, *y1 as f32),
                    ctrl2: Point::new(*x2 as f32, *y2 as f32),
                    to: self.prev,
                });
            }
            Some(PathSegment::ClosePath) => {
                self.needs_end = false;
                self.prev = self.first;
                return_event = Some(PathEvent::End {
                    last: self.prev,
                    first: self.first,
                    close: true,
                });
            }
            None => {
                if self.needs_end {
                    self.needs_end = false;
                    let last = self.prev;
                    let first = self.first;
                    return_event = Some(PathEvent::End {
                        last,
                        first,
                        close: false,
                    });
                }
            }
        }

        return return_event.map(|event| event.transformed(&self.scale));
    }
}

/// Convert an SVG color to a Bevy color.
fn svg_color_to_bevy(paint: &Paint, opacity: u8) -> [f32; 4] {
    return match paint {
        Paint::Color(color) => dbg!(Color::rgba_u8(color.red, color.green, color.blue, opacity)),
        // We only support plain colors
        _ => Color::default(),
    }
    .as_rgba_f32();
}
