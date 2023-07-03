use std::f32::consts::PI;

use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_flycam::prelude::*;
use noise::{NoiseFn, Perlin};

const SEED: u32 = 1;

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .add_system(bevy::window::close_on_esc)
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 12.0,          // default: 12.0
        })
        .add_system(setup.on_startup())
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Directional Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .into(),
        ..default()
    });

    // info!("Move camera around by using WASD for lateral movement");
    // info!("Use Left Shift and Spacebar for vertical movement");
    // info!("Use the mouse to look around");
    // info!("Press Esc to hide or show the mouse cursor");

    // spawn a bunch of cubes
    for x in -100..=100 {
        for z in -100..=100 {
            spawn_column(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(x as f32, z as f32),
            );
        }
    }
}

fn spawn_cube(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(position.x, position.y, position.z),
        ..Default::default()
    });
}

fn spawn_column(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec2, // height is generated from a noise function
) {
    let perlin = Perlin::new(SEED);
    let val = perlin.get([position.x as f64 / 64., position.y as f64 / 64.]) as f32;
    // Round to 1 decimal places
    let max_height = (val * 50.0).round() / 50.0;
    let max_height = max_height.abs() * 10.0;
    // info!("Height: {}", height);

    // Spawn a cube from bottom to top
    for height in 0..=max_height as i32 {
        spawn_cube(
            commands,
            meshes,
            materials,
            Vec3::new(position.x, height as f32, position.y),
        );
    }
}

// If there are two or more verticies at the exact same position, remove until there is only one
