use bevy::{prelude::{Plugin, Assets, HandleUntyped, App, Handle, AddAsset, Msaa}, render::{render_resource::{Shader, SpecializedPipelines}, render_component::{ExtractComponentPlugin, UniformComponentPlugin}, render_asset::RenderAssetPlugin, RenderApp, RenderStage, render_phase::AddRenderCommand}, core_pipeline::{Opaque3d, AlphaMask3d, Transparent3d}, reflect::TypeUuid};

use crate::{VoxelVolume, DrawVoxels, VoxelPipeline, DEFAULT_VOXEL_VOLUME_HANDLE, VoxelVolumeUniform};

pub const VOXEL_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2557421741759925429);
pub const DEPTH_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2007950517632262887);

/// Sets up the Voxel render infrastructure
#[derive(Default)]
pub struct VoxelVolumePlugin;

impl Plugin for VoxelVolumePlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        shaders.set_untracked(
            VOXEL_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("voxel.wgsl")),
        );
        shaders.set_untracked(
            DEPTH_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("depth.wgsl")),
        );

        app.insert_resource(Msaa { samples: 1 });

        app.add_asset::<VoxelVolume>()
            .add_plugin(ExtractComponentPlugin::<Handle<VoxelVolume>>::default())
            .add_plugin(RenderAssetPlugin::<VoxelVolume>::default());

        app.world
            .get_resource_mut::<Assets<VoxelVolume>>()
            .unwrap()
            .set_untracked(
                DEFAULT_VOXEL_VOLUME_HANDLE,
                VoxelVolume::new([0, 0, 0]),
            );

        app.add_plugin(UniformComponentPlugin::<VoxelVolumeUniform>::default());

        let render_app = app.sub_app(RenderApp);
        render_app
            // .add_render_command::<Opaque3d, DrawVoxels>()
            // .add_render_command::<AlphaMask3d, DrawVoxels>()
            .init_resource::<VoxelPipeline>()
            .init_resource::<SpecializedPipelines<VoxelPipeline>>()
            .add_system_to_stage(RenderStage::Extract, super::voxel::extract_voxel_volumes)
            .add_system_to_stage(RenderStage::Queue, super::voxel::queue_voxel_volume_view_bind_groups)
            .add_system_to_stage(RenderStage::Queue, super::voxel::queue_voxel_volume_uniform_bind_groups)
            .add_system_to_stage(RenderStage::Queue, super::voxel::queue_voxel_volumes);

        render_app.add_render_command::<Transparent3d, DrawVoxels>();
    }
}
