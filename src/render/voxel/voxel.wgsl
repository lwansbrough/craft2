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
    [[align(16)]] resolution: vec3<f32>;
    [[align(16)]] size: vec3<f32>;
    palette: array<u32, 256>;
    indirection_pool: array<IndirectionGrid>;
};

struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
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

let COLOR_RED_MASK = 0x000000FFu;
let COLOR_GREEN_MASK = 0x0000FF00u;
let COLOR_BLUE_MASK = 0x00FF0000u;
let COLOR_ALPHA_MASK = 0xFF000000u;

let CELL_TYPE_MASK: u32 = 0x000000FFu;
let CELL_DATA_MASK: u32 = 0xFFFFFF00u;

fn get_voxel(pos_in: vec3<f32>) -> vec4<f32> {
    let color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var pos = vec3<u32>(pos_in);
    
    var pool_index = 0u;
    var grid_size = u32(max(max(voxel_volume.size.x, voxel_volume.size.y), voxel_volume.size.z));
    var grid_cell_size = grid_size / 2u;
    let max_depth: i32 = i32(log2(f32(grid_size)));
    
    for (var i: i32 = 0; i < max_depth; i = i + 1) {
        let grid = &voxel_volume.indirection_pool[pool_index];
        let grid_coord_x = u32(pos.x / grid_cell_size);
        let grid_coord_y = u32(pos.y / grid_cell_size);
        let grid_coord_z = u32(pos.z / grid_cell_size);
        let grid_index = grid_coord_x + grid_coord_y * 2u + grid_coord_z * 2u * 2u;
        let cell = (*grid).cells[grid_index].data;
        let cell_type = (cell & CELL_TYPE_MASK);

        switch (cell_type) {
            case 1u: {
                pool_index = (cell & CELL_DATA_MASK) >> 8u;
                grid_cell_size = grid_cell_size / 2u;
                pos = vec3<u32>(
                    pos.x - grid_coord_x * grid_cell_size,
                    pos.y - grid_coord_y * grid_cell_size,
                    pos.z - grid_coord_z * grid_cell_size
                );
            }
            case 2u: {
                let palette_index = (cell & CELL_DATA_MASK) >> 8u;
                let palette_color = voxel_volume.palette[palette_index];

                return vec4<f32>(1.0, 0.0, 1.0, 1.0);
                
                // return vec4<f32>(
                //     f32(palette_color & COLOR_ALPHA_MASK) / 255.0
                //     f32((palette_color & COLOR_BLUE_MASK) >> 8u) / 255.0,
                //     f32((palette_color & COLOR_GREEN_MASK) >> 16u) / 255.0,
                //     f32((palette_color & COLOR_RED_MASK) >> 24u) / 255.0,
                // );
            }
            default: {
                // discard;
                return vec4<f32>(f32(grid_coord_x) / f32(grid_cell_size), f32(grid_coord_y) / f32(grid_cell_size), f32(grid_coord_z) / f32(grid_cell_size), 1.0);
                // return vec4<f32>(f32(pos.x) / voxel_volume.size.x, f32(pos.y) / voxel_volume.size.y, f32(pos.z) / voxel_volume.size.z, 1.0);
            }
        }
    }

    // return vec4<f32>(0.0, 0.0, 1.0, 1.0);

    discard;
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

    // Convert the local space position into voxel space, ie. [-1, 1] -> [0, 32]
    let voxel_position = model_front_face_pos * voxel_volume.size / world_size;

    // return vec4<f32>(floor(voxel_position) / voxel_volume.size, 1.0);

    let ray_dir = model_ray_dir;
    let ray_dir_len = length(ray_dir);
    let ray_position = voxel_position + 0.0001 * ray_dir;
    var map_pos = floor(ray_position);

    let delta_dist = abs(vec3<f32>(ray_dir_len, ray_dir_len, ray_dir_len) / ray_dir);
	let ray_step = vec3<f32>(sign(ray_dir));
	var side_dist = (sign(ray_dir) * (map_pos - ray_position) + (sign(ray_dir) * 0.5) + 0.5) * delta_dist; 
	
	var mask: vec3<bool>;
    var color: vec4<f32> = vec4<f32>(1.0, 0.0, 1.0, 1.0);

	for (var i: i32 = 0; i < 512; i = i + 1) {
        if (any(map_pos >= voxel_volume.size)) {
            color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
            break;
        }
        if (any(map_pos < vec3<f32>(0.0))) {
            color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
            break;
        }

        color = get_voxel(map_pos);
        
		if (color.a != 0.0) {
            break;
        }

		mask = side_dist.xyz <= min(side_dist.yzx, side_dist.zxy);
        side_dist = side_dist + vec3<f32>(mask) * delta_dist;
        map_pos = map_pos + vec3<f32>(mask) * ray_step;
	}
	
	if (mask.x) {
		color = color * vec4<f32>(vec3<f32>(0.5), 1.0);
	}
	if (mask.y) {
		color = color * vec4<f32>(vec3<f32>(1.0), 1.0);
	}
	if (mask.z) {
		color = color * vec4<f32>(vec3<f32>(0.75), 1.0);
	}
    
    // // return vec4<f32>(1.0, 0.0, 1.0, 1.0);
    return color;
}
