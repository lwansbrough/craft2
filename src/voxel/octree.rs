use std::{cell::RefCell, collections::VecDeque, rc::Rc};
use serde::Serialize;

use crate::{u24_to_bytes, bytes_to_u24};

/// https://developer.nvidia.com/gpugems/gpugems2/part-v-image-oriented-computing/chapter-37-octree-textures-gpu
#[derive(Clone, Debug, Serialize)]
pub struct Octree {
    #[serde(skip)]
    depth_max: u8,
    #[serde(skip)]
    free_indices: VecDeque<u32>,
    indirection_pool: Vec<IndirectionGrid>,
    // released_grids: VecDeque<?>
}

impl Octree {
    pub fn new(depth_max: u8) -> Octree {
        let mut pool = Vec::with_capacity(1);
        pool.push(IndirectionGrid::default());
        Octree {
            depth_max,
            indirection_pool: pool,
            free_indices: VecDeque::with_capacity(1),
        }
    }

    pub fn add_data(&mut self, mut x: u8, mut y: u8, mut z: u8, data: [u8; 3]) {
        let mut pool_index = 0u32;

        let mut depth = 0u8;
        while depth < self.depth_max {
            let grid = &self.indirection_pool[pool_index as usize];
            let grid_cell_size = 2u16.pow(u32::from(self.depth_max - grid.depth)) / 2;
            let grid_x = (x as u16 / grid_cell_size) as u8;
            let grid_y = (y as u16 / grid_cell_size) as u8;
            let grid_z = (z as u16 / grid_cell_size) as u8;

            let cell_index = u32::from(grid_x + grid_y * 2 + grid_z * 2 * 2);

            let mut cell = grid.cells[cell_index as usize];
            
            match cell.cell_type {
                GridCellType::Empty => {
                    if depth == self.depth_max - 1 {
                        cell.cell_type = GridCellType::Material;
                        cell.data = data;
                        self.update_grid_cell(pool_index, cell_index, cell);
                        return;
                    } else {
                        let child_pool_index = self.create_grid_child(pool_index, grid_x, grid_y, grid_z);
                        pool_index = child_pool_index;
                    }
                },
                GridCellType::GridPointer => {
                    pool_index = bytes_to_u24(cell.data);
                },
                GridCellType::Material => {
                    cell.data = data;
                    self.update_grid_cell(pool_index, cell_index, cell);
                    return;
                },
                _ => {}
            }
            
            // pool_index = u32::from(u16::from(pool_offsets[0]) + (u16::from(pool_offsets[1]) * 256u16) + (u16::from(pool_offsets[2]) * 256 * 256));
            x -= (grid_x as u16 * grid_cell_size) as u8;
            y -= (grid_y as u16 * grid_cell_size) as u8;
            z -= (grid_z as u16 * grid_cell_size) as u8;
            depth += 1;
        }
    }

    fn root(&mut self) -> &mut IndirectionGrid {
        &mut self.indirection_pool[0]
    }

    fn create_grid_child(&mut self, pool_index: u32, grid_x: u8, grid_y: u8, grid_z: u8) -> u32 {
        let grid = &self.indirection_pool[pool_index as usize].clone();

        let depth = grid.depth + 1;

        if depth > self.depth_max {
            panic!("max tree depth exceeded!");
        }

        let child_pool_index = self.free_indices.pop_front().unwrap_or_else(|| {
           self.indirection_pool.reserve(1);
           self.indirection_pool.len() as u32
        });

        let child_grid = IndirectionGrid::new(depth);
        self.indirection_pool.insert(child_pool_index as usize, child_grid);

        let mut grid_cells = grid.cells;

        grid_cells[usize::from(grid_x + grid_y * 2 + grid_z * 2 * 2)] = GridCell::new(
            GridCellType::GridPointer,
            u24_to_bytes(child_pool_index)
        );

        self.indirection_pool[pool_index as usize].cells = grid_cells;

        // self.indirection_pool[pool_index as usize] = IndirectionGrid {
        //     depth: grid.depth,
        //     grid_coord: grid.grid_coord,
        //     pool_coord: grid.pool_coord,
        //     cells: grid_cells
        // };

        child_pool_index
        // TODO: set update
    }

    fn update_grid_cell(&mut self, pool_index: u32, cell_index: u32, cell: GridCell) {
        let grid = &mut self.indirection_pool[pool_index as usize];
        grid.cells[cell_index as usize] = cell;
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        for grid in &self.indirection_pool {
            for cell in &grid.cells {
                unsafe {
                    let cell_bytes = ::std::slice::from_raw_parts(
                        (cell as *const GridCell) as *const u8,
                        ::std::mem::size_of::<GridCell>(),
                    );
                    bytes.extend_from_slice(cell_bytes);
                }
            }
        }

        bytes
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct IndirectionGrid {
    #[serde(skip)]
    depth: u8,
    cells: [GridCell; 8]
}

impl Default for IndirectionGrid {
    fn default() -> IndirectionGrid {
        IndirectionGrid {
            depth: 0,
            cells: [
                GridCell::default(),
                GridCell::default(),
                GridCell::default(),
                GridCell::default(),
                GridCell::default(),
                GridCell::default(),
                GridCell::default(),
                GridCell::default()
            ]
        }
    }
}

impl IndirectionGrid {
    pub fn new(depth: u8) -> IndirectionGrid {
        IndirectionGrid {
            depth,
            ..Default::default()
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize)]
#[repr(C)]
pub struct GridCell {
    cell_type: GridCellType,
    data: [u8; 3]
}

impl Default for GridCell {
    fn default() -> GridCell {
        GridCell {
            cell_type: GridCellType::Empty,
            data: [0u8; 3]
        }
    }
}

impl GridCell {
    pub fn new(cell_type: GridCellType, data: [u8; 3]) -> GridCell {
        GridCell {
            cell_type,
            data
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize)]
#[repr(u8)]
pub enum GridCellType {
    Empty,
    GridPointer,
    Material,
    Attachment, // A child octree pointer + connection orientation (1 of 24 -- 6 faces * 4 orientations per face)
}
