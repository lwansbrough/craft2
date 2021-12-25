use bevy::{reflect::TypeUuid, math::{Vec3, Mat4}, render::{render_asset::{RenderAsset, PrepareAssetError}, render_resource::{Buffer, BindGroup, BufferInitDescriptor, BufferUsages, BindGroupDescriptor, BindGroupEntry}, renderer::RenderDevice}, ecs::system::{lifetimeless::SRes, SystemParamItem}, core::{cast_slice, bytes_of}, prelude::{HandleUntyped, Component}};
use crevice::std140::{AsStd140};

use crate::VoxelPipeline;

pub const DEFAULT_VOXEL_VOLUME_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(VoxelVolume::TYPE_UUID, 12003909316817809417);

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "623cfd62-039d-4c2b-b0d4-ed523e87bc6e"]
pub struct VoxelVolume {
    pub resolution: f32,
    pub size: Vec3,
    pub palette: [u32; 255],
    pub data: Vec<u32>
}

impl VoxelVolume {
    pub fn new(size: [u32; 3]) -> Self {
        VoxelVolume::with_resolution(size, 16)
    }

    pub fn with_resolution(size: [u32; 3], voxels_per_meter: u32) -> Self {
        VoxelVolume {
            resolution: 1.0f32 / (voxels_per_meter as f32),
            size: Vec3::new(size[0] as f32, size[1] as f32, size[2] as f32),
            palette: [0;  255],
            data: Vec::new()
        }
    }
}

impl VoxelVolume {
    pub fn to_bytes(&self) -> Vec<u8> {
        let resolution_vec = Vec3::splat(self.resolution);
        let resolution_bytes = bytes_of(&resolution_vec);
        let size_bytes = bytes_of(&self.size);
        let palette_bytes = cast_slice(self.palette.as_slice());
        let data_bytes = cast_slice(self.data.as_slice());

        let mut buffer = vec![0; self.byte_len()];

        let mut offset = 0;
        buffer[offset..resolution_bytes.len()].copy_from_slice(resolution_bytes);

        offset += resolution_bytes.len();
        buffer[offset..(offset + size_bytes.len())].copy_from_slice(size_bytes);

        offset += size_bytes.len();
        buffer[offset..(offset + palette_bytes.len())].copy_from_slice(palette_bytes);

        offset += palette_bytes.len();
        buffer[offset..(offset + data_bytes.len())].copy_from_slice(data_bytes);

        buffer
    }

    pub fn byte_len(&self) -> usize {
        std::mem::size_of::<Vec3>() +
        std::mem::size_of::<Vec3>() +
        std::mem::size_of::<u32>() * self.palette.len() +
        std::mem::size_of::<u32>() * self.data.len()
    }
}

/// The GPU representation of the uniform data of a [`VoxelVolume`].
#[derive(Component, Clone, AsStd140)]
pub struct VoxelVolumeUniform {
    pub transform: Mat4,
    pub inverse_transpose_model: Mat4,
}

/// The index info of a [`GpuVoxelVolume`].
#[derive(Debug, Clone)]
pub struct GpuIndexInfo {
    /// Contains all index data of a mesh.
    pub buffer: Buffer,
    pub count: u32
}

/// The GPU representation of a [`VoxelVolume`].
#[derive(Debug, Clone)]
pub struct GpuVoxelVolume {
    pub vertex_buffer: Buffer,
    /// A buffer containing the [`VoxelVolumeBufferData`] of the volume.
    pub buffer: Buffer,
    /// The bind group specifying how the [`VoxelVolumeUniformData`] and [`VoxelVolumeBufferData`] are bound.
    pub bind_group: BindGroup,
    pub index_info: Option<GpuIndexInfo>
}

impl RenderAsset for VoxelVolume {
    type ExtractedAsset = VoxelVolume;
    type PreparedAsset = GpuVoxelVolume;
    type Param = (SRes<RenderDevice>, SRes<VoxelPipeline>);
    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, voxel_pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let mut vertices: Vec<[f32; 3]> = Vec::new();
        vertices.push([-1.0, 1.0, 0.0]);
        vertices.push([-1.0, -1.0, 0.0]);
        vertices.push([1.0, -1.0, 0.0]);
        vertices.push([1.0, 1.0, 0.0]);
        
        let mut indices: Vec<u16> = Vec::new();
        indices.push(0);
        indices.push(1);
        indices.push(2);
        indices.push(0);
        indices.push(3);
        indices.push(2);

        let vertex_buffer_data = cast_slice(&vertices);
        let index_buffer_data = cast_slice(&indices);

        let vertex_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            usage: BufferUsages::VERTEX,
            label: None,
            contents: &vertex_buffer_data
        });

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: extracted_asset.to_bytes().as_slice(),
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        let index_info = Some(GpuIndexInfo {
            buffer: render_device.create_buffer_with_data(&BufferInitDescriptor {
                usage: BufferUsages::INDEX,
                contents: index_buffer_data,
                label: None,
            }),
            count: 6
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