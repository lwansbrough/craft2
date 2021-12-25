use bevy::{prelude::{Bundle, Handle, Transform, GlobalTransform}, render::{mesh::Mesh, view::{Visibility, ComputedVisibility}}};

use crate::{VoxelVolume, DEFAULT_VOXEL_VOLUME_HANDLE};

/// A component bundle for VoxelVolume entities with a [`Mesh`] and a [`VoxelVolume`].
#[derive(Bundle, Clone)]
pub struct VoxelBundle {
    pub volume: Handle<VoxelVolume>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
}

impl Default for VoxelBundle {
    fn default() -> Self {
        Self {
            volume: DEFAULT_VOXEL_VOLUME_HANDLE.typed(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            computed_visibility: Default::default(),
        }
    }
}
