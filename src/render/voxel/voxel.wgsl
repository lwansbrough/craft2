[[block]]
struct View {
    view_proj: mat4x4<f32>;
    inverse_view: mat4x4<f32>;
    projection: mat4x4<f32>;
    world_position: vec3<f32>;
    near: f32;
    far: f32;
    width: f32;
    height: f32;
};

[[block]]
struct VoxelVolumeUniform {
    transform: mat4x4<f32>;
    inverse_transpose_model: mat4x4<f32>;
};

// struct Octree {
//     info: u32;
//     data: array<u32>;
// };

[[block]]
struct VoxelVolume {
    resolution: vec3<f32>;
    size: vec3<f32>;
    palette: array<u32, 256>;
    data: array<u32>;
    // data: Octree;
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
};

[[group(0), binding(0)]]
var<uniform> view: View;

[[group(1), binding(0)]]
var<uniform> voxel_volume_uniform: VoxelVolumeUniform;

[[group(2), binding(0)]]
var<storage, read> voxel_volume: VoxelVolume;

fn get_voxel(pos: vec3<f32>) -> vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
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

    return out;
}

struct FragmentInput {
    [[builtin(front_facing)]] is_front: bool;
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
};

[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {
    let world_size = voxel_volume.size * voxel_volume.resolution;
    let camera_to_model = view.inverse_view * voxel_volume_uniform.transform;
    let model_back_face_pos = in.position.xyz / vec3<f32>(view.width, view.height, 1.0);
    let model_ray_origin = (camera_to_model * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
    let model_ray_dir = normalize(model_back_face_pos - model_ray_origin);
    let center_offset = vec3<f32>(0.5, 0.5, 0.5) * world_size;

    let model_n = -sign(model_ray_origin);
    let d = -center_offset;
    let t = -(model_ray_origin * model_n - d) / (model_ray_dir * model_n);
    let f = sign(floor(abs(model_ray_origin) * 2.0 / world_size));
    let best_t = max(max(t.x * f.x, t.y * f.y), t.z * f.z);
    let best = select(model_back_face_pos, model_ray_origin + best_t * model_ray_dir, f.x > 0.0 || f.y > 0.0 || f.z > 0.0);

    let model_front_face_pos = (best + center_offset);

    // Convert the local space position into voxel space, ie. [-1, 1] -> [0, 32]
    let voxel_position = model_front_face_pos * voxel_volume.size;

    let ray_dir = model_ray_dir;
    let ray_dir_len = length(ray_dir);
    let ray_position = voxel_position + 0.0001 * ray_dir;
    var map_pos = floor(ray_position);
    let delta_dist = abs(vec3<f32>(ray_dir_len, ray_dir_len, ray_dir_len) / ray_dir);
	let ray_step = vec3<f32>(sign(ray_dir));
	var side_dist = (sign(ray_dir) * (map_pos - ray_position) + (sign(ray_dir) * 0.5) + 0.5) * delta_dist; 
	
	var mask: vec3<bool>;
    var color: vec4<f32>;

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

    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
    // return color;
}
