use bevy::{
    core::Time,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::prelude::*,
    input::Input,
    math::{Quat, Vec3},
    pbr::{
        AmbientLight, DirectionalLight, DirectionalLightBundle, PbrBundle, PointLight,
        PointLightBundle, StandardMaterial,
    },
    prelude::{App, Assets, BuildChildren, KeyCode, Transform, Msaa},
    render::{
        camera::{OrthographicProjection, PerspectiveCameraBundle},
        color::Color,
        mesh::{shape, Mesh},
    },
    DefaultPlugins,
};
use craft2::{VoxelVolumePlugin, VoxelVolume, VoxelBundle, color_to_rgba_u32, u24_to_bytes};
use bevy_flycam::{PlayerPlugin, FlyCam, MovementSettings};
use rand::{Rng, distributions::Uniform, distributions::Distribution };


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(VoxelVolumePlugin)
        .add_plugin(PlayerPlugin)
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 10.0, // default: 12.0
        })
        .add_startup_system(setup)
        .run();
}

#[derive(Component)]
struct Movable;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut voxel_volumes: ResMut<Assets<VoxelVolume>>,
) {
    let mut rng = rand::thread_rng();
    let uni = Uniform::from(0.0..=1.0f32);
    let uni2 = Uniform::from(0u32..=255);

    let mut test = VoxelVolume::new([64, 64, 64]);
    for x in 0..=255 {
        let r = uni.sample(&mut rng);
        let g = uni.sample(&mut rng);
        let b = uni.sample(&mut rng);

        test.palette[x] = color_to_rgba_u32(Color::rgba(r, g, b, 1.0));
    }

    for x in 0..=63 {
        for y in 0..=63 {
            for z in 0..=63 {
                let n1: u32 = uni2.sample(&mut rng);
                test.data.add_data(x, y, z, u24_to_bytes(n1));
            }
        }
    }
    
    let test_handle = voxel_volumes.add(test);

    // voxel volume
    for x in 0..16 {
        for z in 0..16 {
            commands.spawn_bundle(VoxelBundle {
                transform: Transform::from_xyz((x * 4) as f32, 1.0, (z * 4) as f32),
                volume: test_handle.clone(),
                ..Default::default()
            });
        }
    }
    
    

    // // sphere
    // commands.spawn_bundle(PbrBundle {
    //     transform: Transform::from_xyz(2.0, 2.0, 0.0),
    //     mesh: meshes.add(Mesh::from(shape::Icosphere { radius: 0.5, subdivisions: 5 })),
    //     material: materials.add(StandardMaterial {
    //         base_color: Color::BLUE,
    //         perceptual_roughness: 1.0,
    //         ..Default::default()
    //     }),
    //     ..Default::default()
    // });

    // ground plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, -1.0, 0.0),
        ..Default::default()
    });

    // commands.spawn_bundle(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Box::new(0.25, 0.25, 0.25))),
    //     transform: Transform::from_xyz(1.125, 0.875, 1.125),
    //     material: materials.add(StandardMaterial {
    //         base_color: Color::INDIGO,
    //         perceptual_roughness: 1.0,
    //         ..Default::default()
    //     }),
    //     ..Default::default()
    // });

    // // left wall
    // let mut transform = Transform::from_xyz(2.5, 2.5, 0.0);
    // transform.rotate(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
    // commands.spawn_bundle(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Box::new(5.0, 0.15, 5.0))),
    //     transform,
    //     material: materials.add(StandardMaterial {
    //         base_color: Color::INDIGO,
    //         perceptual_roughness: 1.0,
    //         ..Default::default()
    //     }),
    //     ..Default::default()
    // });
    // // back (right) wall
    // let mut transform = Transform::from_xyz(0.0, 2.5, -2.5);
    // transform.rotate(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
    // commands.spawn_bundle(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Box::new(5.0, 0.15, 5.0))),
    //     transform,
    //     material: materials.add(StandardMaterial {
    //         base_color: Color::INDIGO,
    //         perceptual_roughness: 1.0,
    //         ..Default::default()
    //     }),
    //     ..Default::default()
    // });

    // // cube
    // commands
    //     .spawn_bundle(PbrBundle {
    //         mesh: meshes.add(Mesh::from(shape::Box::new(16.0, 16.0, 16.0))),
    //         material: materials.add(StandardMaterial {
    //             base_color: Color::PINK,
    //             ..Default::default()
    //         }),
    //         transform: Transform::from_xyz(0.0, 0.5, 0.0),
    //         ..Default::default()
    //     })
    //     .insert(Movable);
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 1.0, 1.0))),
            material: materials.add(StandardMaterial {
                base_color: Color::PINK,
                ..Default::default()
            }),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .insert(Movable);
    // // sphere
    // commands
    //     .spawn_bundle(PbrBundle {
    //         mesh: meshes.add(Mesh::from(shape::UVSphere {
    //             radius: 0.5,
    //             ..Default::default()
    //         })),
    //         material: materials.add(StandardMaterial {
    //             base_color: Color::LIME_GREEN,
    //             ..Default::default()
    //         }),
    //         transform: Transform::from_xyz(1.5, 1.0, 1.5),
    //         ..Default::default()
    //     })
    //     .insert(Movable);

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::ORANGE_RED,
        brightness: 0.02,
    });

    // // red point light
    // commands
    //     .spawn_bundle(PointLightBundle {
    //         // transform: Transform::from_xyz(5.0, 8.0, 2.0),
    //         transform: Transform::from_xyz(1.0, 2.0, 0.0),
    //         point_light: PointLight {
    //             intensity: 1600.0, // lumens - roughly a 100W non-halogen incandescent bulb
    //             color: Color::RED,
    //             shadows_enabled: true,
    //             ..Default::default()
    //         },
    //         ..Default::default()
    //     })
    //     .with_children(|builder| {
    //         builder.spawn_bundle(PbrBundle {
    //             mesh: meshes.add(Mesh::from(shape::UVSphere {
    //                 radius: 0.1,
    //                 ..Default::default()
    //             })),
    //             material: materials.add(StandardMaterial {
    //                 base_color: Color::RED,
    //                 emissive: Color::rgba_linear(100.0, 0.0, 0.0, 0.0),
    //                 ..Default::default()
    //             }),
    //             ..Default::default()
    //         });
    //     });

    // // green point light
    // commands
    //     .spawn_bundle(PointLightBundle {
    //         // transform: Transform::from_xyz(5.0, 8.0, 2.0),
    //         transform: Transform::from_xyz(-1.0, 2.0, 0.0),
    //         point_light: PointLight {
    //             intensity: 1600.0, // lumens - roughly a 100W non-halogen incandescent bulb
    //             color: Color::GREEN,
    //             shadows_enabled: true,
    //             ..Default::default()
    //         },
    //         ..Default::default()
    //     })
    //     .with_children(|builder| {
    //         builder.spawn_bundle(PbrBundle {
    //             mesh: meshes.add(Mesh::from(shape::UVSphere {
    //                 radius: 0.1,
    //                 ..Default::default()
    //             })),
    //             material: materials.add(StandardMaterial {
    //                 base_color: Color::GREEN,
    //                 emissive: Color::rgba_linear(0.0, 100.0, 0.0, 0.0),
    //                 ..Default::default()
    //             }),
    //             ..Default::default()
    //         });
    //     });

    // // blue point light
    // commands
    //     .spawn_bundle(PointLightBundle {
    //         // transform: Transform::from_xyz(5.0, 8.0, 2.0),
    //         transform: Transform::from_xyz(0.0, 4.0, 0.0),
    //         point_light: PointLight {
    //             intensity: 1600.0, // lumens - roughly a 100W non-halogen incandescent bulb
    //             color: Color::BLUE,
    //             shadows_enabled: true,
    //             ..Default::default()
    //         },
    //         ..Default::default()
    //     })
    //     .with_children(|builder| {
    //         builder.spawn_bundle(PbrBundle {
    //             mesh: meshes.add(Mesh::from(shape::UVSphere {
    //                 radius: 0.1,
    //                 ..Default::default()
    //             })),
    //             material: materials.add(StandardMaterial {
    //                 base_color: Color::BLUE,
    //                 emissive: Color::rgba_linear(0.0, 0.0, 100.0, 0.0),
    //                 ..Default::default()
    //             }),
    //             ..Default::default()
    //         });
    //     });

    // directional 'sun' light
    const HALF_SIZE: f32 = 10.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..Default::default()
            },
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..Default::default()
        },
        ..Default::default()
    });

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        // transform: Transform::from_xyz(0.0, 1.0, -10.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..Default::default()
    })
    .insert(FlyCam);
}
