pub mod render;
pub mod voxel;
// pub mod worldgen;

use bevy::prelude::Color;

pub use self::{
    render::*,
    voxel::*,
    // worldgen::*
};

pub fn u24_to_bytes(v: u32) -> [u8; 3] {
    [
        (v & 0x000000ff) as u8,
        ((v & 0x0000ff00) >> 8) as u8,
        ((v & 0x00ff0000) >> 16) as u8
    ]
}

pub fn bytes_to_u24(v: [u8; 3]) -> u32 {
    let mut num = 0u32;
    
    num |= (v[0] as u32) << 0;
    num |= (v[1] as u32) << 8;
    num |= (v[2] as u32) << 16;
    
    num
}

pub fn color_to_rgba_u32(color: Color) -> u32 {
    let r = (color.r() * 255.0) as u8;
    let g = (color.g() * 255.0) as u8;
    let b = (color.b() * 255.0) as u8;
    let a = (color.a() * 255.0) as u8;
    let rp = (u32::from(r)) << 24;
    let gp = (u32::from(g)) << 16;
    let bp = (u32::from(b)) << 8;
    let ap = u32::from(a);
    
    rp | gp | bp | ap
}
