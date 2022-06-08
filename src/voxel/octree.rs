use std::{cell::RefCell, collections::VecDeque, rc::Rc};
use serde::Serialize;

use crate::{u24_to_bytes, bytes_to_u24};

/// https://developer.nvidia.com/gpugems/gpugems2/part-v-image-oriented-computing/chapter-37-octree-textures-gpu
#[derive(Clone, Debug, Serialize)]
pub struct Octree {
    #[serde(skip)]
    grids_max: [u8; 3],
    #[serde(skip)]
    grids_next_free: [u8; 3],
    #[serde(skip)]
    depth_max: u8,
    indirection_pool: Vec<IndirectionGrid>,
    // released_grids: VecDeque<?>
}

impl Octree {
    pub fn new(depth_max: u8) -> Octree {
        let grids_max: [u8; 3] = [(2u16.pow(u32::from(depth_max))) as u8; 3];
        let mut pool = Vec::with_capacity(1);
        pool.push(IndirectionGrid::default());
        Octree {
            grids_max,
            indirection_pool: pool,
            grids_next_free: [0, 0, 0],
            depth_max
        }
    }

    pub fn add_data(&mut self, mut x: u8, mut y: u8, mut z: u8, data: [u8; 3]) {
        let mut pool_index = 0u32;

        // let mut depth = 0;
        // while depth < self.depth_max {
        //     let grid = &self.indirection_pool[pool_index];
        //     let grid_cell_size = (2u16.pow(u32::from(self.depth_max - grid.depth)) / 2 - 1) as u8;

        //     let grid_x = x / grid_cell_size;
        //     let grid_y = y / grid_cell_size;
        //     let grid_z = z / grid_cell_size;

        //     x = x - grid_x * grid_cell_size;
        //     y = y - grid_y * grid_cell_size;
        //     z = z - grid_z * grid_cell_size;

        //     let cell_index = u32::from(grid_x + grid_y * 2 + grid_z * 2 * 2);
        //     let mut cell = grid.cells[cell_index];

        //     println!("({:?}, {:?}, {:?}) {:?}", x, y, z, cell_index);

        //     depth += 1;
        // }

        let mut depth = 0;
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

    fn compute_offset(&self, node_pool_coord: [u8; 3], mut node_grid_coord: [u8; 3], grid_size: u8) -> [u8; 3] {
        node_grid_coord[0] %= grid_size;
        node_grid_coord[1] %= grid_size;
        node_grid_coord[2] %= grid_size;

        let (ti, tj, tk, di, dj, dk): (u8, u8, u8, u8, u8, u8);

        ti = node_grid_coord[0] % self.grids_max[0];
        tj = node_grid_coord[1] % self.grids_max[1];
        tk = node_grid_coord[2] % self.grids_max[2];
        di = node_pool_coord[0] - ti;
        dj = node_pool_coord[1] - tj;
        dk = node_pool_coord[2] - tk;

        [di as u8 & 255, dj as u8 & 255, dk as u8 & 255]
    }

    fn create_grid_child(&mut self, pool_index: u32, grid_x: u8, grid_y: u8, grid_z: u8) -> u32 {
        let grid = &self.indirection_pool[pool_index as usize].clone();
        let grid_size_next = 2u8.pow(grid.depth as u32 + 1);
        let grid_coord = grid.grid_coord();
        let grid_pos = [
            grid_coord[0] * 2 + grid_x,
            grid_coord[1] * 2 + grid_y,
            grid_coord[2] * 2 + grid_z
        ];

        let depth = grid.depth + 1;
        let grid_coord = grid_pos;
        let mut child_index = 0;

        if depth > self.depth_max {
            panic!("max tree depth exceeded!");
        }

        if self.grids_next_free[2] < self.grids_max[2] {
            self.indirection_pool.reserve(1);

            if self.grids_next_free[0] < self.grids_max[0] - 1 {
                self.indirection_pool.reserve(1);
                self.grids_next_free[0] += 1;
            } else {
                if self.grids_next_free[1] < self.grids_max[1] - 1 {
                    self.indirection_pool.reserve(1);
                    self.grids_next_free[0] = 0;
                    self.grids_next_free[1] += 1;
                } else {
                    self.grids_next_free[0] = 0;
                    self.grids_next_free[1] = 0;
                    self.grids_next_free[2] += 1;
                }
            }

            let next_free = self.grids_next_free.clone();
            let child_grid = IndirectionGrid::new(depth, grid_coord, next_free);
            let pool_index = u32::from(u16::from(next_free[0]) + (u16::from(next_free[1]) * u16::from(self.grids_max[1])) + (u16::from(next_free[2]) * u16::from(self.grids_max[2]) * u16::from(self.grids_max[1])));
            self.indirection_pool.insert(pool_index as usize, child_grid);

            child_index = pool_index;
        }

        // panic!("idk");
        // } else {
        //     let removed = self.released_grids.pop_front().unwrap();

        // }

        let child = &self.indirection_pool[child_index as usize];
        // let offset = self.compute_offset(child.pool_coord(), grid_pos, grid_size_next);

        let mut grid_cells = grid.cells;

        grid_cells[usize::from(grid_x + grid_y * 2 + grid_z * 2 * 2)] = GridCell::new(
            GridCellType::GridPointer,
            u24_to_bytes(child_index)
        );

        self.indirection_pool[pool_index as usize] = IndirectionGrid {
            depth: grid.depth,
            grid_coord: grid.grid_coord,
            pool_coord: grid.pool_coord,
            cells: grid_cells
        };

        child_index
        // TODO: set update
    }

    fn update_grid_cell(&mut self, pool_index: u32, cell_index: u32, cell: GridCell) {
        let grid = &mut self.indirection_pool[pool_index as usize];
        grid.cells[cell_index as usize] = cell;
    }

    pub fn set_grid(&mut self, offset: [u8; 3], grid: IndirectionGrid) {
        let index =  u32::from(u16::from(offset[0]) + (u16::from(offset[1]) * 256u16) + (u16::from(offset[2]) * 256 * 256));
        self.indirection_pool[index as usize] = grid;
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
    #[serde(skip)]
    grid_coord: [u8; 3],
    #[serde(skip)]
    pool_coord: [u8; 3],
    cells: [GridCell; 8]
}

impl Default for IndirectionGrid {
    fn default() -> IndirectionGrid {
        IndirectionGrid {
            depth: 0,
            grid_coord: [0; 3],
            pool_coord: [0; 3],
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
    pub fn new(depth: u8, grid_coord: [u8; 3], pool_coord: [u8; 3]) -> IndirectionGrid {
        IndirectionGrid {
            depth,
            grid_coord,
            pool_coord,
            ..Default::default()
        }
    }

    pub fn grid_coord(&self) -> [u8; 3] {
        self.grid_coord
    }

    pub fn pool_coord(&self) -> [u8; 3] {
        self.pool_coord
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
