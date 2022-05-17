struct View {
    view_proj: mat4x4<f32>;
    view: mat4x4<f32>;
    inverse_view: mat4x4<f32>;
    projection: mat4x4<f32>;
    world_position: vec3<f32>;
    near: f32;
    far: f32;
    width: f32;
    height: f32;
};

struct VoxelVolumeUniform {
    transform: mat4x4<f32>;
    inverse_transform: mat4x4<f32>;
    inverse_transpose_model: mat4x4<f32>;
};

struct GridCell {
    data: u32;
};

struct IndirectionGrid {
    cells: array<GridCell, 8>;
};

struct VoxelVolume {
    [[size(16)]] resolution: vec3<f32>;
    [[size(16)]] size: vec3<f32>;
    palette: array<u32, 256>;
    indirection_pool: array<IndirectionGrid>;
};

struct Vertex {
    [[location(1)]] position: vec3<f32>;
    [[location(0)]] normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
    [[location(3)]] vertex_position: vec3<f32>;
};

[[group(0), binding(0)]]
var<uniform> view: View;

[[group(1), binding(0)]]
var<uniform> voxel_volume_uniform: VoxelVolumeUniform;

[[group(2), binding(0)]]
var<storage, read> voxel_volume: VoxelVolume;

let COLOR_RED_MASK = 0xFF000000u;
let COLOR_GREEN_MASK = 0x00FF0000u;
let COLOR_BLUE_MASK = 0x0000FF00u;
let COLOR_ALPHA_MASK = 0x000000FFu;

let CELL_TYPE_MASK: u32 = 0x000000FFu;
let CELL_DATA_MASK: u32 = 0xFFFFFF00u;

let CELL_TYPE_GRID_POINTER = 1u;
let CELL_TYPE_DATA = 2u;
let CELL_TYPE_EMPTY = 0u;

// fn get_voxel(pos_in: vec3<f32>) -> vec4<f32> {
//     let color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
//     var pos = vec3<u32>(pos_in);
    
//     var pool_index = 0u;
//     var grid_size = u32(max(max(voxel_volume.size.x, voxel_volume.size.y), voxel_volume.size.z));
//     var grid_cell_size = grid_size / 2u;
//     let max_depth: i32 = i32(log2(f32(grid_size)));
    
//     for (var i: i32 = 0; i < max_depth; i = i + 1) {
//         let grid = &voxel_volume.indirection_pool[pool_index];
//         let grid_coord_x = u32(pos.x / grid_cell_size);
//         let grid_coord_y = u32(pos.y / grid_cell_size);
//         let grid_coord_z = u32(pos.z / grid_cell_size);
//         let grid_index = grid_coord_x + grid_coord_y * 2u + grid_coord_z * 2u * 2u;
//         let cell = (*grid).cells[grid_index].data;
//         let cell_type = (cell & CELL_TYPE_MASK);

//         switch (cell_type) {
//             case 1u: {
//             // case CELL_TYPE_GRID_POINTER: {
//                 pool_index = (cell & CELL_DATA_MASK) >> 8u;
//                 grid_cell_size = grid_cell_size / 2u;
//                 pos = vec3<u32>(
//                     pos.x - grid_coord_x * grid_cell_size,
//                     pos.y - grid_coord_y * grid_cell_size,
//                     pos.z - grid_coord_z * grid_cell_size
//                 );
//             }
//             case 2u: {
//             // case CELL_TYPE_DATA: {
//                 let palette_index = (cell & CELL_DATA_MASK) >> 8u;
//                 let palette_color = voxel_volume.palette[palette_index];

//                 let alpha = f32(palette_color & COLOR_ALPHA_MASK) / 255.0;
//                 let blue = f32((palette_color & COLOR_BLUE_MASK) >> 8u) / 255.0;
//                 let green = f32((palette_color & COLOR_GREEN_MASK) >> 16u) / 255.0;
//                 let red = f32((palette_color & COLOR_RED_MASK) >> 24u) / 255.0;

//                 return vec4<f32>(
//                     red,
//                     green,
//                     blue,
//                     alpha
//                 );
//             }
//             default: {
//                 // discard;
//                 // return vec4<f32>(f32(grid_coord_x) / f32(grid_cell_size), f32(grid_coord_y) / f32(grid_cell_size), f32(grid_coord_z) / f32(grid_cell_size), 1.0);
//                 return vec4<f32>(f32(pos.x) / voxel_volume.size.x, f32(pos.y) / voxel_volume.size.y, f32(pos.z) / voxel_volume.size.z, 1.0);
//             }
//         }
//     }

//     // return vec4<f32>(0.0, 0.0, 1.0, 1.0);

//     discard;
// }

// struct Stack {
//     index: u32;
//     center: vec3<f32>;
//     scale: f32;
// }

fn raybox_intersect(box_min: vec3<f32>, box_max: vec3<f32>, ray_dir: vec3<f32>, ray_inv_dir: vec3<f32>, ray_origin: vec3<f32>) -> bool {
	let tbot = ray_inv_dir * (box_min - ray_origin);
	let ttop = ray_inv_dir * (box_max - ray_origin);
	let tmin = min(ttop, tbot);
	let tmax = max(ttop, tbot);
	var t = max(tmin.xx, tmin.yz);
	let t0 = max(t.x, t.y);
	t = min(tmax.xx, tmax.yz);
	let t1 = min(t.x, t.y);
    return t1 > max(t0, 0.0);
}

// fn trace_voxel(ray_dir: vec3<f32>, ray_position: vec3<f32>) -> vec4<f32> {
//     let center = vec3<f32>(0.0, 0.0, 0.0);
//     var scale = 1.0;
//     let min_box = center - scale;
//     let max_box = center + scale;
//     var f = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    
//     var stack = array<Stack, 10>();
//     var stack_pos = 1u;
//     var pool_index = 0u;
//     scale = scale * 0.5;
//     stack[0] = Stack(0u, center, scale);
    
//     while (stack_pos-- > 0) {
//         f = vec4<f32>(0.1, 0.1, 0.1, 0.1);
//         center = stack[stack_pos].center;
//         index = stack[stack_pos].index;
//         scale = stack[stack_pos].scale;

//         let grid = &voxel_volume.indirection_pool[pool_index];
//         var accumulated_offset = 0u;

//         for (var grid_index: u32 = 0u; grid_index < 8u; grid_index++) {
//             let cell = (*grid).cells[grid_index].data;
//             let cell_type = (cell & CELL_TYPE_MASK);

//             let new_center = center + scale * POS[grid_index];
//             let min_box = new_center - scale;
//             let max_box = new_center + scale;

//             if (!raybox_intersect(min_box, max_box, ray_dir, ray_position)) {
//                 if (cell_type != CELL_TYPE_DATA) {
//                     accumulated_offset = accumulated_offset + 1;
//                 }
//                 continue;
//             }

//             if (cell_type == CELL_TYPE_DATA) {
//                 let palette_index = (cell & CELL_DATA_MASK) >> 8u;
//                 let palette_color = voxel_volume.palette[palette_index];

//                 let alpha = f32(palette_color & COLOR_ALPHA_MASK) / 255.0;
//                 let blue = f32((palette_color & COLOR_BLUE_MASK) >> 8u) / 255.0;
//                 let green = f32((palette_color & COLOR_GREEN_MASK) >> 16u) / 255.0;
//                 let red = f32((palette_color & COLOR_RED_MASK) >> 24u) / 255.0;

//                 return vec4<f32>(
//                     red,
//                     green,
//                     blue,
//                     alpha
//                 );
//             } else {

//             }

//             switch (cell_type) {
//                 case CELL_TYPE_EMPTY:
//                     continue;
//                 case CELL_TYPE_GRID_POINTER:
//                     pool_index = (cell & CELL_DATA_MASK) >> 8u;
//                     break;
//                 case CELL_TYPE_DATA: {
                    
//                 }
//             }
//         }
//     }
// }


struct Stack {
    pool_index: u32;
    grid_index: u32;
};

fn trace_voxel(ray_dir: vec3<f32>, ray_position: vec3<f32>, world_size: vec3<f32>) -> vec4<f32> {
    let ray_dir_inv = 1.0 / ray_dir;

    var center = vec3<f32>(0.0, 0.0, 0.0);
    var scale: f32 = 0.5;

    var POS = array<vec3<f32>, 8>(
        vec3<f32>(1.0, 1.0, 1.0),
        vec3<f32>(-1.0, 1.0,1.0),
        vec3<f32>(1.0, -1.0, 1.0),
        vec3<f32>(-1.0, -1.0, 1.0),
        vec3<f32>(1.0, 1.0, -1.0),
        vec3<f32>(-1.0, 1.0, -1.0),
        vec3<f32>(1.0, -1.0, -1.0),
        vec3<f32>(-1.0, -1.0, -1.0)
    );
    
    var stack = array<Stack, 8>(
        Stack(0u, 0u),
        Stack(0u, 0u),
        Stack(0u, 0u),
        Stack(0u, 0u),
        Stack(0u, 0u),
        Stack(0u, 0u),
        Stack(0u, 0u),
        Stack(0u, 0u)
    );
    var stack_pos = 1u;
    var pool_index = 0u;
    var color = vec4<f32>(1.0, 0.0, 1.0, 1.0);

    for (var stack_pos = 1u; stack_pos > 0u; stack_pos = stack_pos - 1u) {
        let grid = &voxel_volume.indirection_pool[pool_index];

        for (var grid_index: u32 = 0u; grid_index < 8u; grid_index = grid_index + 1u) {
            let cell_center = center + scale * POS[grid_index];
            let min_box = cell_center - scale;
            let max_box = cell_center + scale;

            if (!raybox_intersect(min_box, max_box, ray_dir, ray_dir_inv, ray_position)) {
                continue;
            }

            let cell = (*grid).cells[grid_index].data;
            let cell_type = (cell & CELL_TYPE_MASK);

            switch (cell_type) {
                case 0u: {
                // case CELL_TYPE_EMPTY:
                    continue;
                }
                case 1u: {
                // case CELL_TYPE_GRID_POINTER:
                    pool_index = (cell & CELL_DATA_MASK) >> 8u;
                    center = cell_center;
                    scale = scale / 2.0;
                    stack_pos = stack_pos + 1u;
                    color = color * (scale * 1.5);
                    break;
                }
                case 2u: {
                // case CELL_TYPE_DATA: {
                    let palette_index = (cell & CELL_DATA_MASK) >> 8u;
                    let palette_color = voxel_volume.palette[palette_index];

                    let alpha = f32(palette_color & COLOR_ALPHA_MASK) / 255.0;
                    let blue = f32((palette_color & COLOR_BLUE_MASK) >> 8u) / 255.0;
                    let green = f32((palette_color & COLOR_GREEN_MASK) >> 16u) / 255.0;
                    let red = f32((palette_color & COLOR_RED_MASK) >> 24u) / 255.0;

                    return vec4<f32>(
                        red,
                        green,
                        blue,
                        alpha
                    );
                }
                default: {
                    continue;
                }
            }

            break;
        }
    }

    return color;
}

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let world_position = voxel_volume_uniform.transform * vec4<f32>(vertex.position, 1.0);

    out.uv = vertex.uv;
    out.world_position = world_position;
    out.clip_position = view.view_proj * world_position;
    out.world_normal = mat3x3<f32>(
        voxel_volume_uniform.inverse_transpose_model[0].xyz,
        voxel_volume_uniform.inverse_transpose_model[1].xyz,
        voxel_volume_uniform.inverse_transpose_model[2].xyz
    ) * vertex.normal;
    out.vertex_position = vertex.position;

    return out;
}

struct FragmentInput {
    [[builtin(front_facing)]] is_front: bool;
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
    [[location(3)]] vertex_position: vec3<f32>;
};

[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {
    let world_size = voxel_volume.size * voxel_volume.resolution;
    let camera_to_model = voxel_volume_uniform.inverse_transform * view.view;
    let model_back_face_pos = in.vertex_position;
    let model_ray_origin = (camera_to_model * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
    let model_ray_dir = normalize(model_back_face_pos - model_ray_origin);
    let center_offset = vec3<f32>(0.5, 0.5, 0.5) * world_size;

    let model_n = -sign(model_ray_origin);
    let d = -center_offset;
    let t = -(model_ray_origin * model_n - d) / (model_ray_dir * model_n); // division by model_ray_dir here blows t up to a huge number? or maybe model_ray_origin is too big by this point (ie. miscalculated?)
    let f = sign(floor(abs(model_ray_origin) * 2.0 / world_size));
    let best_t = max(max(t.x * f.x, t.y * f.y), t.z * f.z);
    let best = select(model_back_face_pos, model_ray_origin + best_t * model_ray_dir, f.x > 0.0 || f.y > 0.0 || f.z > 0.0);

    let model_front_face_pos = (best + center_offset);

    let color: vec4<f32> = trace_voxel(model_ray_dir, model_front_face_pos, world_size);
    
	
	// if (mask.x) {
	// 	color = color * vec4<f32>(vec3<f32>(0.5), 1.0);
	// }
	// if (mask.y) {
	// 	color = color * vec4<f32>(vec3<f32>(1.0), 1.0);
	// }
	// if (mask.z) {
	// 	color = color * vec4<f32>(vec3<f32>(0.75), 1.0);
	// }
    
    return color;
}
