use anyhow::{Context, Error};
use bevy::asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset};
use bevy::{
    prelude::{Color, Mesh},
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    utils::tracing,
};
use lyon_tessellation::geom::euclid::default::Transform2D;
use lyon_tessellation::{
    math::Point, path::PathEvent, BuffersBuilder, FillOptions, FillTessellator, FillVertex,
    FillVertexConstructor, LineCap, LineJoin, StrokeOptions, StrokeTessellator, StrokeVertex,
    StrokeVertexConstructor, VertexBuffers,
};
use std::slice::Iter;
use usvg::{NodeKind, Options, Paint, Path, PathSegment, Transform, Tree};

/// Lyon tolerance for generating a mesh from the stroke.
pub const STROKE_TOLERANCE: f32 = 0.01;

/// Bevy SVG asset loader.
#[derive(Debug, Default)]
pub struct SvgAssetLoader;

impl AssetLoader for SvgAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            bevy::log::debug!("Loading SVG {:?}", load_context.path());

            // Parse and simplify the SVG file
            let svg_tree = Tree::from_data(&bytes, &Options::default().to_ref())
                .with_context(|| format!("Could not parse SVG file {:?}", load_context.path()))?;

            // Generate the mesh
            let mesh = svg_to_mesh(&svg_tree);

            // Upload the mesh
            load_context.set_default_asset(LoadedAsset::new(mesh));

            bevy::log::trace!("SVG loaded");

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["svg"]
    }
}

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

// Taken from https://github.com/nical/lyon/blob/74e6b137fea70d71d3b537babae22c6652f8843e/examples/wgpu_svg/src/main.rs
struct PathConvIter<'a> {
    iter: Iter<'a, PathSegment>,
    prev: Point,
    first: Point,
    needs_end: bool,
    deferred: Option<PathEvent>,
}

impl<'a> PathConvIter<'a> {
    /// Convert a SVG path to the iterator for the tessellator.
    pub fn from_svg_path(path: &'a Path) -> Self {
        Self {
            iter: path.data.0.iter(),
            first: Point::zero(),
            prev: Point::zero(),
            deferred: None,
            needs_end: false,
        }
    }
}

impl<'l> Iterator for PathConvIter<'l> {
    type Item = PathEvent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.deferred.is_some() {
            return self.deferred.take();
        }

        let next = self.iter.next();
        match next {
            Some(PathSegment::MoveTo { x, y }) => {
                if self.needs_end {
                    let last = self.prev;
                    let first = self.first;
                    self.needs_end = false;
                    self.prev = Point::new((*x) as f32, -(*y) as f32);
                    self.deferred = Some(PathEvent::Begin { at: self.prev });
                    self.first = self.prev;

                    Some(PathEvent::End {
                        last,
                        first,
                        close: false,
                    })
                } else {
                    self.first = Point::new((*x) as f32, -(*y) as f32);

                    Some(PathEvent::Begin { at: self.first })
                }
            }
            Some(PathSegment::LineTo { x, y }) => {
                self.needs_end = true;
                let from = self.prev;
                self.prev = Point::new((*x) as f32, -(*y) as f32);

                Some(PathEvent::Line {
                    from,
                    to: self.prev,
                })
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
                self.prev = Point::new((*x) as f32, -(*y) as f32);

                Some(PathEvent::Cubic {
                    from,
                    ctrl1: Point::new((*x1) as f32, -(*y1) as f32),
                    ctrl2: Point::new((*x2) as f32, -(*y2) as f32),
                    to: self.prev,
                })
            }
            Some(PathSegment::ClosePath) => {
                self.needs_end = false;
                self.prev = self.first;
                Some(PathEvent::End {
                    last: self.prev,
                    first: self.first,
                    close: true,
                })
            }
            None => {
                if self.needs_end {
                    self.needs_end = false;
                    let last = self.prev;
                    let first = self.first;
                    Some(PathEvent::End {
                        last,
                        first,
                        close: false,
                    })
                } else {
                    None
                }
            }
        }
    }
}

/// Convert a SVG to a mesh.
fn svg_to_mesh(svg: &Tree) -> Mesh {
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
                // Convert the usvg stroke options to lyon stroke options
                let linecap = match stroke.linecap {
                    usvg::LineCap::Butt => LineCap::Butt,
                    usvg::LineCap::Round => LineCap::Round,
                    usvg::LineCap::Square => LineCap::Square,
                };
                let linejoin = match stroke.linejoin {
                    usvg::LineJoin::Miter => LineJoin::Miter,
                    usvg::LineJoin::Round => LineJoin::Round,
                    usvg::LineJoin::Bevel => LineJoin::Bevel,
                };

                let stroke_options = StrokeOptions::tolerance(STROKE_TOLERANCE)
                    .with_line_width(stroke.width.value() as f32)
                    .with_line_cap(linecap)
                    .with_line_join(linejoin);

                let mut buffers = svg_path_stroke_to_vertex_buffers(
                    &path,
                    vertices.len() as u32,
                    &stroke_options,
                );

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
    stroke_options: &StrokeOptions,
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
        stroke_options,
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

    let triangles = indices.len() / 3;

    // Create the mesh
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    // Set the indices
    mesh.set_indices(Some(Indices::U32(indices)));

    // Set the vertices
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    // Set the colors
    mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, colors);

    bevy::log::debug!("Mesh created with {triangles} triangles");

    mesh
}

/// Convert an SVG color to a Bevy color.
fn svg_color_to_bevy(paint: &Paint, opacity: u8) -> [f32; 4] {
    return match paint {
        Paint::Color(color) => Color::rgba_u8(color.red, color.green, color.blue, opacity),
        // We only support plain colors
        _ => Color::default(),
    }
    .as_linear_rgba_f32();
}
