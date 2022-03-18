use crate::inspector::Inspectable;
use bevy::{
    core::FloatOrd,
    core_pipeline::Transparent2d,
    prelude::{
        App, Assets, Bundle, Commands, Component, ComputedVisibility, Entity, FromWorld,
        GlobalTransform, Handle, HandleUntyped, Local, Mesh, Msaa, Plugin, Query, Res, ResMut,
        Shader, Transform, Visibility, With, World,
    },
    reflect::TypeUuid,
    render::{
        render_asset::RenderAssets,
        render_phase::{AddRenderCommand, DrawFunctions, RenderPhase, SetItemPipeline},
        render_resource::{
            BlendState, ColorTargetState, ColorWrites, FragmentState, FrontFace, MultisampleState,
            PolygonMode, PrimitiveState, RenderPipelineCache, RenderPipelineDescriptor,
            SpecializedPipeline, SpecializedPipelines, TextureFormat, VertexAttribute,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        texture::BevyDefault,
        view::VisibleEntities,
        RenderApp, RenderStage,
    },
    sprite::{
        DrawMesh2d, Mesh2dHandle, Mesh2dPipeline, Mesh2dPipelineKey, Mesh2dUniform,
        SetMesh2dBindGroup, SetMesh2dViewBindGroup,
    },
};

use crate::geometry::transform::TransformBuilder;

/// Handle to the custom shader with a unique random ID.
pub const COLORED_MESH_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 13518715925425351754);

/// A marker component for colored 2d meshes.
#[derive(Debug, Default, Component, Inspectable)]
pub struct ColoredMesh;

/// Bundle for easy construction of colored meshes.
#[derive(Default, Bundle, Inspectable)]
pub struct ColoredMeshBundle {
    pub colored_mesh: ColoredMesh,
    #[inspectable(ignore)]
    pub handle: Mesh2dHandle,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    #[inspectable(ignore)]
    pub visibility: Visibility,
    #[inspectable(ignore)]
    pub computed_visibility: ComputedVisibility,
}

impl ColoredMeshBundle {
    /// Create a new bundle.
    pub fn new(mesh: Handle<Mesh>) -> Self {
        Self {
            handle: Mesh2dHandle(mesh),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        }
    }
}

impl TransformBuilder for ColoredMeshBundle {
    fn transform_mut_ref(&'_ mut self) -> &'_ mut Transform {
        &mut self.transform
    }
}

/// Custom pipeline for 2d meshes with vertex colors.
pub struct ColoredMeshPipeline {
    /// This pipeline wraps the standard [`Mesh2dPipeline`].
    mesh2d_pipeline: Mesh2dPipeline,
}

impl FromWorld for ColoredMeshPipeline {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh2d_pipeline: Mesh2dPipeline::from_world(world),
        }
    }
}

// We implement `SpecializedPipeline` to customize the default rendering from `Mesh2dPipeline`.
impl SpecializedPipeline for ColoredMeshPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // Customize how to store the meshes' vertex attributes in the vertex buffer
        // Our meshes only have position and color
        let vertex_attributes = vec![
            // Position (GOTCHA! Vertex_Position isn't first in the buffer due to how Mesh sorts attributes (alphabetically))
            VertexAttribute {
                format: VertexFormat::Float32x3,
                // this offset is the size of the color attribute, which is stored first
                offset: 16,
                // position is available at location 0 in the shader
                shader_location: 0,
            },
            // Color
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 1,
            },
        ];
        // This is the sum of the size of position and color attributes (12 + 16 = 28)
        let vertex_array_stride = 28;

        RenderPipelineDescriptor {
            vertex: VertexState {
                // Use our custom shader
                shader: COLORED_MESH_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "vertex".into(),
                shader_defs: Vec::new(),
                // Use our custom vertex buffer
                buffers: vec![VertexBufferLayout {
                    array_stride: vertex_array_stride,
                    step_mode: VertexStepMode::Vertex,
                    attributes: vertex_attributes,
                }],
            },
            fragment: Some(FragmentState {
                // Use our custom shader
                shader: COLORED_MESH_SHADER_HANDLE.typed::<Shader>(),
                shader_defs: Vec::new(),
                entry_point: "fragment".into(),
                targets: vec![ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            // Use the two standard uniforms for 2d meshes
            layout: Some(vec![
                // Bind group 0 is the view uniform
                self.mesh2d_pipeline.view_layout.clone(),
                // Bind group 1 is the mesh uniform
                self.mesh2d_pipeline.mesh_layout.clone(),
            ]),
            primitive: PrimitiveState {
                front_face: FrontFace::Cw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: key.primitive_topology(),
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("colored_mesh_pipeline".into()),
        }
    }
}

/// Specify how to render a colored 2d mesh.
type DrawColoredMesh = (
    // Set the pipeline
    SetItemPipeline,
    // Set the view uniform as bind group 0
    SetMesh2dViewBindGroup<0>,
    // Set the mesh uniform as bind group 1
    SetMesh2dBindGroup<1>,
    // Draw the mesh
    DrawMesh2d,
);

/// Plugin that renders [`ColoredMesh`]s.
pub struct ColoredMeshPlugin;

impl Plugin for ColoredMeshPlugin {
    fn build(&self, app: &mut App) {
        // Load our custom shader
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        shaders.set_untracked(
            COLORED_MESH_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("colored_mesh.wgsl")),
        );

        // Register our custom draw function and pipeline, and add our render systems
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app
            .add_render_command::<Transparent2d, DrawColoredMesh>()
            .init_resource::<ColoredMeshPipeline>()
            .init_resource::<SpecializedPipelines<ColoredMeshPipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_colored_mesh)
            .add_system_to_stage(RenderStage::Queue, queue_colored_mesh);
    }
}

/// Extract the [`ColoredMesh`] marker component into the render app
pub fn extract_colored_mesh(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    query: Query<(Entity, &ComputedVisibility), With<ColoredMesh>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, computed_visibility) in query.iter() {
        if !computed_visibility.is_visible {
            continue;
        }
        values.push((entity, (ColoredMesh,)));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

/// Queue the 2d meshes marked with [`ColoredMesh`] using our custom pipeline and draw function.
#[allow(clippy::too_many_arguments)]
pub fn queue_colored_mesh(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    colored_mesh_pipeline: Res<ColoredMeshPipeline>,
    mut pipelines: ResMut<SpecializedPipelines<ColoredMeshPipeline>>,
    mut pipeline_cache: ResMut<RenderPipelineCache>,
    msaa: Res<Msaa>,
    render_meshes: Res<RenderAssets<Mesh>>,
    colored_mesh: Query<(&Mesh2dHandle, &Mesh2dUniform), With<ColoredMesh>>,
    mut views: Query<(&VisibleEntities, &mut RenderPhase<Transparent2d>)>,
) {
    if colored_mesh.is_empty() {
        return;
    }
    // Iterate each view (a camera is a view)
    for (visible_entities, mut transparent_phase) in views.iter_mut() {
        let draw_colored_mesh = transparent_draw_functions
            .read()
            .get_id::<DrawColoredMesh>()
            .unwrap();

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples);

        // Queue all entities visible to that view
        for visible_entity in &visible_entities.entities {
            if let Ok((mesh2d_handle, mesh2d_uniform)) = colored_mesh.get(*visible_entity) {
                // Get our specialized pipeline
                let mut mesh2d_key = mesh_key;
                if let Some(mesh) = render_meshes.get(&mesh2d_handle.0) {
                    mesh2d_key |=
                        Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);
                }

                let pipeline_id =
                    pipelines.specialize(&mut pipeline_cache, &colored_mesh_pipeline, mesh2d_key);

                let mesh_z = mesh2d_uniform.transform.w_axis.z;
                transparent_phase.add(Transparent2d {
                    entity: *visible_entity,
                    draw_function: draw_colored_mesh,
                    pipeline: pipeline_id,
                    // The 2d render items are sorted according to their z value before rendering,
                    // in order to get correct transparency
                    sort_key: FloatOrd(mesh_z),
                    // This material is not batched
                    batch_range: None,
                });
            }
        }
    }
}
