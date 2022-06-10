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
    [[location(0)]] normal: vec3<f32>;
    [[location(1)]] position: vec3<f32>;
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

struct Intersection {
    hit: bool;
    // point: vec3<f32>;
    distance: f32;
};

fn raybox_intersect(box_min: vec3<f32>, box_max: vec3<f32>, ray_dir: vec3<f32>, ray_inv_dir: vec3<f32>, ray_origin: vec3<f32>) -> Intersection {
	let tbot = ray_inv_dir * (box_min - ray_origin);
	let ttop = ray_inv_dir * (box_max - ray_origin);
	let tmin = min(ttop, tbot);
	let tmax = max(ttop, tbot);
	var traverse = max(tmin.xx, tmin.yz);
	let traverse_near = max(traverse.x, traverse.y);
	traverse = min(tmax.xx, tmax.yz);
	let traverse_far = min(traverse.x, traverse.y);
    return Intersection(traverse_far > max(traverse_near, 0.0), traverse_near);
}

let COLOR_RED_MASK = 0xFF000000u;
let COLOR_GREEN_MASK = 0x00FF0000u;
let COLOR_BLUE_MASK = 0x0000FF00u;
let COLOR_ALPHA_MASK = 0x000000FFu;

let CELL_TYPE_MASK: u32 = 0x000000FFu;
let CELL_DATA_MASK: u32 = 0xFFFFFF00u;

let CELL_TYPE_GRID_POINTER = 1u;
let CELL_TYPE_DATA = 2u;
let CELL_TYPE_EMPTY = 0u;

struct Stack {
    pool_index: u32;
    grid_index: u32;
    depth: u32;
    center: vec3<f32>;
};

struct TraceResult {
    color: vec4<f32>;
    distance: f32;
};

fn trace_voxel(ray_dir: vec3<f32>, ray_position: vec3<f32>) -> TraceResult {
    let ray_dir_inv = 1.0 / ray_dir;

    var POS = array<vec3<f32>, 8>(
        vec3<f32>(-1.0, -1.0, 1.0),
        vec3<f32>(1.0, -1.0, 1.0),
        vec3<f32>(-1.0, 1.0, 1.0),
        vec3<f32>(1.0, 1.0, 1.0),
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>(1.0, -1.0, -1.0),
        vec3<f32>(-1.0, 1.0, -1.0),
        vec3<f32>(1.0, 1.0, -1.0),  
    );
    
    var stack = array<Stack, 8>(
        Stack(0u, 0u, 1u, vec3<f32>(0.0, 0.0, 0.0)),
        Stack(0u, 0u, 2u, vec3<f32>(0.0, 0.0, 0.0)),
        Stack(0u, 0u, 3u, vec3<f32>(0.0, 0.0, 0.0)),
        Stack(0u, 0u, 4u, vec3<f32>(0.0, 0.0, 0.0)),
        Stack(0u, 0u, 5u, vec3<f32>(0.0, 0.0, 0.0)),
        Stack(0u, 0u, 6u, vec3<f32>(0.0, 0.0, 0.0)),
        Stack(0u, 0u, 7u, vec3<f32>(0.0, 0.0, 0.0)),
        Stack(0u, 0u, 8u, vec3<f32>(0.0, 0.0, 0.0))
    );

    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var hit_dist = 1000000000.0;
    var hit_depth = 0u;
    var curr_dist = 0.0;

    for (var stack_pos: u32 = 1u; stack_pos > 0u; stack_pos = stack_pos - 1u) {
        let stack_index = stack_pos - 1u;
        let stack_entry = &stack[stack_index];
        let pool_index = (*stack_entry).pool_index;
        let grid_index = (*stack_entry).grid_index;
        let center = (*stack_entry).center;
        let depth = (*stack_entry).depth;
        
        let scale = 1.0 / pow(2.0, f32(depth));
        let grid = &voxel_volume.indirection_pool[pool_index];

        // var max_depth = u32(floor(8.0 * min(1.0, 100.0 / curr_dist) + 1.0));

        // if (depth == max_depth) {

        // }
        // else {}

        for (var curr_grid_index: u32 = grid_index; curr_grid_index < 8u; curr_grid_index = curr_grid_index + 1u) {
            let cell_center = center + scale * POS[curr_grid_index];
            var min_box = cell_center - vec3<f32>(scale);
            var max_box = cell_center + vec3<f32>(scale);

            let intersection = raybox_intersect(min_box, max_box, ray_dir, ray_dir_inv, ray_position);
            
            if (!intersection.hit || intersection.distance > hit_dist) {
                continue;
            }

            curr_dist = intersection.distance;

            let cell = (*grid).cells[curr_grid_index].data;
            let cell_type = (cell & CELL_TYPE_MASK);

            switch (cell_type) {
                case 0u: {
                // case CELL_TYPE_EMPTY:
                    continue;
                }
                case 1u: {
                // case CELL_TYPE_GRID_POINTER:
                    let next_pool_index = (cell & CELL_DATA_MASK) >> 8u;

                    (*stack_entry).grid_index = curr_grid_index + 1u;

                    let next_stack_entry = &stack[stack_index + 1u];
                    (*next_stack_entry).pool_index = next_pool_index;
                    (*next_stack_entry).grid_index = 0u;
                    (*next_stack_entry).depth = depth + 1u;
                    (*next_stack_entry).center = cell_center;

                    stack_pos = stack_pos + 2u;
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

                    hit_dist = intersection.distance;
                    color = vec4<f32>(
                        red,
                        green,
                        blue,
                        alpha
                    );

                    continue;
                }
                default: {
                    continue;
                }
            }

            break;
        }
    }

    return TraceResult(color, hit_dist);
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

struct FragmentOutput {
    [[location(0)]] color: vec4<f32>;
    // [[builtin(frag_depth)]] depth: f32;
};

[[stage(fragment)]]
fn fragment(in: FragmentInput) -> FragmentOutput {
    let world_size = voxel_volume.size * voxel_volume.resolution;
    let half_world_size = world_size / 2.0;
    // let model_world_position = voxel_volume_uniform.transform[3].xyz;
    let camera_to_model = voxel_volume_uniform.inverse_transform * view.view;
    let model_back_face_pos = in.vertex_position;
    let model_ray_origin = (camera_to_model * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
    let model_ray_dir = normalize(model_back_face_pos - model_ray_origin);
    let center_offset = half_world_size;

    let model_n = -sign(model_ray_origin);
    let d = -center_offset;
    let t = -(model_ray_origin * model_n - d) / (model_ray_dir * model_n); // division by model_ray_dir here blows t up to a huge number? or maybe model_ray_origin is too big by this point (ie. miscalculated?)
    let f = sign(floor(abs(model_ray_origin) * 2.0 / world_size));
    let best_t = max(max(t.x * f.x, t.y * f.y), t.z * f.z);
    let best = select(model_back_face_pos, model_ray_origin + best_t * model_ray_dir, f.x > 0.0 || f.y > 0.0 || f.z > 0.0);

    let model_front_face_pos = best / half_world_size; // [-1, 1]
    let model_front_face_ray_dir = normalize(model_front_face_pos - model_ray_origin);

    let result = trace_voxel(model_front_face_ray_dir, model_front_face_pos);

    // return FragmentOutput(result.color, result.distance);
    return FragmentOutput(result.color);
}
