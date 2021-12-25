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

[[block]]
struct VoxelVolume {
    resolution: vec3<f32>;
    size: vec3<f32>;
    palette: array<u32, 255>;
    data: array<u32>;
};

struct Vertex {
    [[location(0)]] position: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> view: View;

[[group(1), binding(0)]]
var<uniform> voxel_volume_uniform: VoxelVolumeUniform;

[[group(2), binding(0)]]
var<storage, read> voxel_volume: VoxelVolume;

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    
    var voxel_volume_world_size = voxel_volume.size * voxel_volume.resolution;

    var quadric_matrix: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, -1.0);
    var sphere_radius: f32 = max(voxel_volume_world_size.x, max(voxel_volume_world_size.y, voxel_volume_world_size.z)) * 1.732051;

    var sphere_center: vec4<f32> = voxel_volume_uniform.transform[3];
    var sphere_center_world = sphere_center * voxel_volume_uniform.transform;
    var model_view_proj: mat4x4<f32> = voxel_volume_uniform.transform * view.view_proj;
    var sphere_center_clip = model_view_proj * sphere_center;
    out.clip_position = sphere_center_clip;

    var mat_t: mat3x4<f32> = mat3x4<f32>(
        vec4<f32>(
            model_view_proj[0].x * sphere_radius,
            model_view_proj[0].y * sphere_radius,
            model_view_proj[0].z * sphere_radius,
            dot(sphere_center, model_view_proj[0])
        ),
        vec4<f32>(
            model_view_proj[1].x * sphere_radius,
            model_view_proj[1].y * sphere_radius,
            model_view_proj[1].z * sphere_radius,
            dot(sphere_center, model_view_proj[1])
        ),
        vec4<f32>(
            model_view_proj[3].x * sphere_radius,
            model_view_proj[3].y * sphere_radius,
            model_view_proj[3].z * sphere_radius,
            dot(sphere_center, model_view_proj[3])
        )
    );

    var mat_d: mat3x4<f32> = mat3x4<f32>(
        mat_t[0] * quadric_matrix, 
        mat_t[1] * quadric_matrix,
        mat_t[2] * quadric_matrix
    );

    var eq_coefs: vec4<f32> = vec4<f32>(
        dot(mat_d[0], mat_t[2]),
        dot(mat_d[1], mat_t[2]),
        dot(mat_d[0], mat_t[0]),
        dot(mat_d[1], mat_t[1])
    ) / dot(mat_d[2], mat_t[2]);

    var out_position: vec4<f32> = vec4<f32>(eq_coefs.x, eq_coefs.y, 0.0, 1.0);

    var aabb: vec2<f32> = sqrt((eq_coefs.xy * eq_coefs.xy) - eq_coefs.zw);

    // out.clip_position.x = (out_position.x - (vertex.position.x * (aabb.x / 2.0))) * out.clip_position.w;
    // out.clip_position.y = (out_position.y - (vertex.position.y * (aabb.y / 2.0))) * out.clip_position.w;

    out.clip_position.x = (out_position.x + (sign(vertex.position.x) * 0.05 * out_position.x)) * out.clip_position.w;
    out.clip_position.y = (out_position.y + (sign(vertex.position.y) * 0.05 * out_position.y)) * out.clip_position.w;

    return out;
}

// struct FragmentInput {
//     [[builtin(front_facing)]] is_front: bool;
//     [[location(0)]] world_position: vec4<f32>;
//     [[location(1)]] world_normal: vec3<f32>;
//     [[location(2)]] uv: vec2<f32>;
// };

// [[stage(fragment)]]
// fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {
//     return vec4<f32>(1.0, 0.0, 0.0, 1.0);
// }

[[stage(fragment)]]
fn fragment(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}