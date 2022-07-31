use bevy::{reflect::TypeUuid, math::{Vec3, Mat4}, render::{render_asset::{RenderAsset, PrepareAssetError}, render_resource::{Buffer, BindGroup, BufferInitDescriptor, BufferUsages, BindGroupDescriptor, BindGroupEntry, IndexFormat, ShaderType}, renderer::RenderDevice}, ecs::system::{lifetimeless::SRes, SystemParamItem}, core::{cast_slice, bytes_of}, prelude::{HandleUntyped, Component, ResMut, Assets, Mesh, shape}};

use crate::{VoxelPipeline, Octree};

pub const DEFAULT_VOXEL_VOLUME_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(VoxelVolume::TYPE_UUID, 12003909316817809417);

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "623cfd62-039d-4c2b-b0d4-ed523e87bc6e"]
pub struct VoxelVolume {
    pub resolution: f32,
    pub size: Vec3,
    pub palette: [u32; 256],
    pub data: Octree,
    pub mesh: Mesh
}

impl VoxelVolume {
    pub fn new(size: [u32; 3]) -> Self {
        VoxelVolume::with_resolution(size, 16)
    }

    pub fn with_resolution(size: [u32; 3], voxels_per_meter: u32) -> Self {
        let max_size = size[0].max(size[1]).max(size[2]);
        let resolution = 1.0f32 / (voxels_per_meter as f32);

        VoxelVolume {
            resolution,
            size: Vec3::new(size[0] as f32, size[1] as f32, size[2] as f32),
            palette: [0;  256],
            data: Octree::new((max_size as f32).log2() as u8),
            mesh: Mesh::from(shape::Box::new(
                resolution * (size[0] as f32),
                resolution * (size[1] as f32),
                resolution * (size[2] as f32)
            ))
        }
    }
}

impl VoxelVolume {
    pub fn to_bytes(&self) -> Vec<u8> {
        let resolution_vec = Vec3::splat(self.resolution);
        let resolution_bytes = bytes_of(&resolution_vec);
        let resolution_len = 16; // aligned length
        let size_bytes = bytes_of(&self.size);
        let size_len = 16;
        let palette_bytes = cast_slice(self.palette.as_slice());
        let palette_len = 1024;
        let data = &self.data.to_bytes();
        let data_bytes = cast_slice(data.as_slice());
        let byte_len = resolution_len + size_len + palette_len + data_bytes.len();

        let mut buffer = vec![0; byte_len];

        let mut offset = 0;
        buffer[offset..resolution_bytes.len()].copy_from_slice(resolution_bytes);

        offset += resolution_len;
        buffer[offset..(offset + size_bytes.len())].copy_from_slice(size_bytes);

        offset += size_len;
        buffer[offset..(offset + palette_bytes.len())].copy_from_slice(palette_bytes);

        offset += palette_len;
        buffer[offset..(offset + data_bytes.len())].copy_from_slice(data_bytes);

        buffer
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }
}

/// The GPU representation of the uniform data of a [`VoxelVolume`].
#[derive(Component, Clone, ShaderType)]
pub struct VoxelVolumeUniform {
    pub transform: Mat4,
    pub inverse_transform: Mat4,
    pub inverse_transpose_model: Mat4,
}

/// The index info of a [`GpuVoxelVolume`].
#[derive(Debug, Clone)]
pub enum GpuBufferInfo {
    Indexed {
        /// Contains all index data of a mesh.
        buffer: Buffer,
        count: u32,
        index_format: IndexFormat,
    },
    NonIndexed {
        vertex_count: u32,
    },
}

/// The GPU representation of a [`VoxelVolume`].
#[derive(Debug, Clone)]
pub struct GpuVoxelVolume {
    pub vertex_buffer: Buffer,
    /// A buffer containing the [`VoxelVolumeBufferData`] of the volume.
    pub buffer: Buffer,
    /// The bind group specifying how the [`VoxelVolumeUniformData`] and [`VoxelVolumeBufferData`] are bound.
    pub bind_group: BindGroup,
    pub index_info: GpuBufferInfo
}

impl RenderAsset for VoxelVolume {
    type ExtractedAsset = VoxelVolume;
    type PreparedAsset = GpuVoxelVolume;
    type Param = (SRes<RenderDevice>, SRes<VoxelPipeline>);
    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        voxel_volume: Self::ExtractedAsset,
        (render_device, voxel_pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let mesh = voxel_volume.mesh();

        let vertex_buffer_data = mesh.get_vertex_buffer_data();
        let vertex_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            usage: BufferUsages::VERTEX,
            label: None,
            contents: &vertex_buffer_data
        });

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: voxel_volume.to_bytes().as_slice(),
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        let index_info = mesh.get_index_buffer_bytes().map_or(
            GpuBufferInfo::NonIndexed {
                vertex_count: mesh.count_vertices() as u32
            },
            |data| GpuBufferInfo::Indexed {
            buffer: render_device.create_buffer_with_data(&BufferInitDescriptor {
                usage: BufferUsages::INDEX,
                contents: data,
                label: None,
            }),
            count: mesh.indices().unwrap().len() as u32,
            index_format: mesh.indices().unwrap().into(),
        });

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: None,
            layout: &voxel_pipeline.voxel_layout,
        });

        Ok(GpuVoxelVolume {
            vertex_buffer,
            buffer,
            bind_group,
            index_info
        })
    }
}