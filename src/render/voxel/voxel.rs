use bevy::{render::{render_phase::{SetItemPipeline, EntityRenderCommand, TrackedRenderPass, RenderCommandResult, DrawFunctions, RenderPhase}, render_resource::{BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindingType, BufferBindingType, SpecializedPipeline, RenderPipelineDescriptor, Shader, RenderPipelineCache, SpecializedPipelines, IndexFormat, PrimitiveTopology, PolygonMode, PrimitiveState, FrontFace, VertexState, VertexBufferLayout, ColorTargetState, TextureFormat, ColorWrites, DepthStencilState, CompareFunction, StencilState, StencilFaceState, DepthBiasState, FragmentState, VertexStepMode, MultisampleState, VertexAttribute, VertexFormat, BlendState, Face, BindGroup, BindGroupEntry, BindGroupDescriptor, BufferSize}, renderer::RenderDevice, render_asset::RenderAssets, view::{Msaa, ExtractedView, VisibleEntities, ViewUniform, ViewUniforms, ViewUniformOffset}, mesh::Mesh, texture::BevyDefault, render_component::{ComponentUniforms, DynamicUniformIndex}}, prelude::{FromWorld, World, Handle, Entity, Res, ResMut, Query, With, GlobalTransform, ComputedVisibility, Local, Commands, Component}, ecs::system::{lifetimeless::{SRes, SQuery, Read}, SystemParamItem}, core_pipeline::Transparent3d};
use crevice::std140::AsStd140;

use crate::{VOXEL_SHADER_HANDLE, VoxelVolume, VoxelVolumeUniform, GpuBufferInfo};

#[derive(Clone)]
pub struct VoxelPipeline {
    pub view_layout: BindGroupLayout,
    pub voxel_uniform_layout: BindGroupLayout,
    pub voxel_layout: BindGroupLayout,
}

impl FromWorld for VoxelPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();

        let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(ViewUniform::std140_size_static() as u64),
                    },
                    count: None,
                }
            ],
            label: Some("voxel_view_layout")
        });

        let voxel_uniform_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(VoxelVolumeUniform::std140_size_static() as u64),
                    },
                    count: None,
                }
            ],
            label: Some("voxel_uniform_layout")
        });

        let voxel_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                       ty: BufferBindingType::Storage {
                           read_only: true
                       },
                       has_dynamic_offset: false,
                       min_binding_size: None
                    },
                    count: None,
                }
            ],
            label: Some("voxel_layout"),
        });

        VoxelPipeline {
            view_layout,
            voxel_uniform_layout,
            voxel_layout
        }
    }
}


#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct VoxelPipelineKey;

impl SpecializedPipeline for VoxelPipeline {
    type Key = VoxelPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {

        let mut shader_defs = Vec::new();

        let vertex_array_stride = 32;
        let vertex_attributes = vec![
            // Position (GOTCHA! Vertex_Position isn't first in the buffer due to how Mesh sorts attributes (alphabetically))
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 12,
                shader_location: 0,
            },
            // Normal
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 0,
                shader_location: 1,
            },
            // Uv
            VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: 24,
                shader_location: 2,
            },
        ];

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: VOXEL_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "vertex".into(),
                shader_defs: shader_defs.clone(),
                buffers: vec![VertexBufferLayout {
                    array_stride: vertex_array_stride,
                    step_mode: VertexStepMode::Vertex,
                    attributes: vertex_attributes,
                }],
            },
            fragment: Some(FragmentState {
                shader: VOXEL_SHADER_HANDLE.typed::<Shader>(),
                shader_defs,
                entry_point: "fragment".into(),
                targets: vec![ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            layout: Some(vec![self.view_layout.clone(), self.voxel_uniform_layout.clone(), self.voxel_layout.clone()]),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Front),
                polygon_mode: PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Greater,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            label: Some("voxel_pipeline".into()),
            multisample: MultisampleState::default()
        }
    }
}

pub type DrawVoxels = (
    SetItemPipeline,
    SetVoxelVolumeViewBindGroup<0>,
    SetVoxelVolumeUniformBindGroup<1>,
    SetVoxelBindGroup<2>,
    DrawVoxel,
);

pub struct VoxelBindGroup {
    pub value: BindGroup,
}

#[derive(Component)]
pub struct VoxelVolumeViewBindGroup {
    pub value: BindGroup,
}

pub struct VoxelVolumeUniformBindGroup {
    pub value: BindGroup,
}

pub fn queue_voxel_volume_view_bind_groups(
    mut commands: Commands,
    voxel_pipeline: Res<VoxelPipeline>,
    render_device: Res<RenderDevice>,
    view_uniforms: Res<ViewUniforms>,
    mut views: Query<Entity, With<ExtractedView>>
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        for entity in views.iter_mut() {
            let view_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: view_binding.clone(),
                    }
                ],
                label: Some("voxel_volume_bind_group"),
                layout: &voxel_pipeline.view_layout,
            });

            commands.entity(entity).insert(VoxelVolumeViewBindGroup {
                value: view_bind_group,
            });
        }
    }
}

pub fn queue_voxel_volume_uniform_bind_groups(
    mut commands: Commands,
    voxel_pipeline: Res<VoxelPipeline>,
    render_device: Res<RenderDevice>,
    voxel_uniforms: Res<ComponentUniforms<VoxelVolumeUniform>>
) {
    if let Some(uniform_binding) = voxel_uniforms.uniforms().binding() {
        let uniform_bind_group = VoxelVolumeUniformBindGroup {
            value: render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: uniform_binding,
                    },
                ],
                label: Some("voxel_volume_uniform_bind_group"),
                layout: &voxel_pipeline.voxel_uniform_layout,
            }),
        };

        commands.insert_resource(uniform_bind_group);
    }
}

pub struct SetVoxelVolumeViewBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetVoxelVolumeViewBindGroup<I> {
    type Param = SQuery<(
        Read<ViewUniformOffset>,
        Read<VoxelVolumeViewBindGroup>,
    )>;
    #[inline]
    fn render<'w>(
        view: Entity,
        _item: Entity,
        view_query: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (view_uniform, voxel_volume_view_bind_group) = view_query.get(view).unwrap();
        pass.set_bind_group(
            I,
            &voxel_volume_view_bind_group.value,
            &[view_uniform.offset],
        );

        RenderCommandResult::Success
    }
}

pub struct SetVoxelVolumeUniformBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetVoxelVolumeUniformBindGroup<I> {
    type Param = (
        SRes<VoxelVolumeUniformBindGroup>,
        SQuery<Read<DynamicUniformIndex<VoxelVolumeUniform>>>,
    );
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (voxel_volume_uniform_bind_group, voxel_volume_uniform_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let uniform_index = voxel_volume_uniform_query.get(item).unwrap();
        pass.set_bind_group(
            I,
            &voxel_volume_uniform_bind_group.into_inner().value,
            &[uniform_index.index()],
        );
        RenderCommandResult::Success
    }
}

pub struct SetVoxelBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetVoxelBindGroup<I> {
    type Param = (
        SRes<RenderAssets<VoxelVolume>>,
        SQuery<Read<Handle<VoxelVolume>>>,
    );
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (voxel_volumes, handle_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let handle = handle_query.get(item).unwrap();
        let voxel_volumes = voxel_volumes.into_inner();
        let voxel_volume = voxel_volumes.get(handle).unwrap();

        pass.set_bind_group(I, &voxel_volume.bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub struct DrawVoxel;
impl EntityRenderCommand for DrawVoxel {
    type Param = (SRes<RenderAssets<VoxelVolume>>, SQuery<Read<Handle<VoxelVolume>>>);
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (voxel_volumes, voxel_volume_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let voxel_volume_handle = voxel_volume_query.get(item).unwrap();
        if let Some(gpu_voxel_volume) = voxel_volumes.into_inner().get(voxel_volume_handle) {
            pass.set_vertex_buffer(0, gpu_voxel_volume.vertex_buffer.slice(..));

            match &gpu_voxel_volume.index_info {
                GpuBufferInfo::Indexed {
                    buffer,
                    index_format,
                    count
                } => {
                    pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                    pass.draw_indexed(0..*count, 0, 0..1);
                }
                GpuBufferInfo::NonIndexed { vertex_count } => {
                    pass.draw(0..*vertex_count, 0..1);
                }
            }
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

pub fn extract_voxel_volumes(
    mut commands: Commands,
    mut previous_uniforms_len: Local<usize>,
    voxel_volumes: Query<(
        Entity,
        &ComputedVisibility,
        &GlobalTransform,
        &Handle<VoxelVolume>,
    )>,
) {
    let mut uniforms = Vec::with_capacity(*previous_uniforms_len);
    for (entity, computed_visibility, transform, handle) in voxel_volumes.iter() {
        if !computed_visibility.is_visible {
            continue;
        }
        let transform = transform.compute_matrix();

        uniforms.push((
            entity,
            (
                handle.clone_weak(),
                VoxelVolumeUniform {
                    transform,
                    inverse_transpose_model: transform.inverse().transpose()
                }
            )
        ));
    }
    *previous_uniforms_len = uniforms.len();
    commands.insert_or_spawn_batch(uniforms);
}

pub fn queue_voxel_volumes(
    transparent_draw_functions: Res<DrawFunctions<Transparent3d>>,
    render_voxel_volumes: Res<RenderAssets<VoxelVolume>>,
    voxel_pipeline: Res<VoxelPipeline>,
    mut pipeline_cache: ResMut<RenderPipelineCache>,
    mut pipelines: ResMut<SpecializedPipelines<VoxelPipeline>>,
    msaa: Res<Msaa>,
    voxel_volumes: Query<(Entity, &Handle<VoxelVolume>, &VoxelVolumeUniform), With<Handle<VoxelVolume>>>,
    mut views: Query<(
        &ExtractedView,
        &VisibleEntities,
        &mut RenderPhase<Transparent3d>,
    )>,
) {
    for (view, visible_entities, mut transparent_phase) in views.iter_mut() {
        let draw_voxels = transparent_draw_functions
            .read()
            .get_id::<DrawVoxels>()
            .unwrap();

        let view_matrix = view.transform.compute_matrix();
        let view_row_2 = view_matrix.row(2);

        for visible_entity in &visible_entities.entities {
            if let Ok((entity, voxel_volume_handle, uniform)) =
                voxel_volumes.get(*visible_entity)
            {
                if let Some(voxel_volume) = render_voxel_volumes.get(voxel_volume_handle) {
                    let key = VoxelPipelineKey;
                    
                    transparent_phase.add(Transparent3d {
                        entity: *visible_entity,
                        pipeline: pipelines.specialize(
                            &mut pipeline_cache,
                            &voxel_pipeline,
                            key,
                        ),
                        draw_function: draw_voxels,
                        distance: view_row_2.dot(uniform.transform.col(3)),
                    });
                }
            }
        }
    }
}
