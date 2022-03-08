use bevy::{
    prelude::Mesh,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use lyon_tessellation::math::Transform;
use lyon_tessellation::{
    path::PathEvent, BuffersBuilder, FillOptions, FillTessellator, FillVertex,
    FillVertexConstructor, StrokeOptions, StrokeTessellator, StrokeVertex, StrokeVertexConstructor,
    VertexBuffers,
};

/// Convert a geo polygon to a mesh.
pub trait ToMesh {
    /// Get the vertices, indices and colors.
    fn buffers(&self) -> (Vec<[f32; 3]>, Vec<u32>, Vec<[f32; 4]>);

    /// Convert the object to a mesh.
    fn to_mesh(&self) -> Mesh {
        bevy::log::trace!("Creating mesh");

        let (vertices, indices, colors) = self.buffers();
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
}

/// Buffers for creating a mesh.
#[derive(Debug, Default)]
pub struct MeshBuffers {
    vertices: Vec<[f32; 3]>,
    indices: Vec<u32>,
    colors: Vec<[f32; 4]>,
}

impl MeshBuffers {
    /// Construct a new buffers object with empty buffers.
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert a path fill to vertex and index buffers.
    pub fn append_fill<C>(
        &mut self,
        path: impl IntoIterator<Item = PathEvent>,
        transform: Transform,
        color: C,
    ) where
        C: Into<[f32; 4]>,
    {
        bevy::log::trace!("Converting path fill to vertex buffers");

        // The resulting vertex and index buffers
        let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();

        // Use our custom vertex constructor to create a bevy vertex buffer
        let mut vertex_builder =
            BuffersBuilder::new(&mut buffers, BevyVertexConstructor { transform });

        // Tesselate the fill
        let mut tessellator = FillTessellator::new();
        let result = tessellator.tessellate(path, &FillOptions::default(), &mut vertex_builder);
        assert!(result.is_ok());

        self.merge_buffers(buffers, color.into());
    }

    /// Convert a path stroke to vertex and index buffers.
    pub fn append_stroke<C>(
        &mut self,
        path: impl IntoIterator<Item = PathEvent>,
        stroke_options: &StrokeOptions,
        transform: Transform,
        color: C,
    ) where
        C: Into<[f32; 4]>,
    {
        bevy::log::trace!("Converting path stroke to vertex buffers");

        // The resulting vertex and index buffers
        let mut buffers: VertexBuffers<[f32; 3], u32> = VertexBuffers::new();

        // Use our custom vertex constructor to create a bevy vertex buffer
        let mut vertex_builder =
            BuffersBuilder::new(&mut buffers, BevyVertexConstructor { transform });

        // Tesselate the fill
        let mut tessellator = StrokeTessellator::new();
        let result = tessellator.tessellate(path, stroke_options, &mut vertex_builder);
        assert!(result.is_ok());

        self.merge_buffers(buffers, color.into());
    }

    /// Merge the buffers.
    fn merge_buffers(&mut self, mut buffers: VertexBuffers<[f32; 3], u32>, color: [f32; 4]) {
        // Add the offset so multiple items can be merged
        let indices_offset = self.vertices.len() as u32;
        if indices_offset != 0 {
            buffers
                .indices
                .iter_mut()
                .for_each(|index| *index += indices_offset);
        }

        self.vertices.append(&mut buffers.vertices);
        self.indices.append(&mut buffers.indices);

        // Fill the buffer with the same size as the vertices with colors
        self.colors.resize(self.vertices.len(), color);
    }
}

impl ToMesh for MeshBuffers {
    fn buffers(&self) -> (Vec<[f32; 3]>, Vec<u32>, Vec<[f32; 4]>) {
        (
            self.vertices.clone(),
            self.indices.clone(),
            self.colors.clone(),
        )
    }
}

/// A custom vertex constructor for lyon, creates bevy vertices.
struct BevyVertexConstructor {
    /// The transform to apply to all vertices.
    transform: Transform,
}

impl FillVertexConstructor<[f32; 3]> for BevyVertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> [f32; 3] {
        // Transform the 2D point
        let transformed = self.transform.transform_point(vertex.position());

        [transformed.x, transformed.y, 0.0]
    }
}

impl StrokeVertexConstructor<[f32; 3]> for BevyVertexConstructor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> [f32; 3] {
        // Transform the 2D point
        let transformed = self.transform.transform_point(vertex.position());

        [transformed.x, transformed.y, 0.0]
    }
}
