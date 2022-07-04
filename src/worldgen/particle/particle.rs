use std::collections::VecDeque;

use bevy::math::{Vec2, IVec2};

pub trait Particle {
    fn pos() -> Vec2;
    fn speed() -> Vec2;
    fn is_alive() -> bool;
    fn transport(map: &LayerMap) -> bool;
    fn interact(map: &LayerMap) -> bool;
}

pub fn cascade(pos: Vec2, map: &LayerMap) {
    let ipos: IVec2 = pos.round().as_ivec2();

    const n: Vec<IVec2> = vec![
        IVec2::new(-1, -1),
        IVec2::new(-1, 0),
        IVec2::new(-1, 1),
        IVec2::new(0, -1),
        IVec2::new(0, 1),
        IVec2::new(1, -1),
        IVec2::new(1, 0),
        IVec2::new(1, 1)
    ];

    let sn: VecDeque<IVec2> = VecDeque::new();
    for nn in n {
        let npos = ipos + nn;
        if npos.x >= map.dim.x || npos.y >= map.dim.y || npos.x < 0 || npos.y < 0 {
            continue;
        }
        sn.push_back(npos);
    }

    
}
