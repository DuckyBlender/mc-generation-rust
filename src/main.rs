use std::f32::consts::PI;

use bevy::{
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_flycam::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use noise::{NoiseFn, Perlin};

const SEED: u32 = 1;
const CHUNK_SIZE: u32 = 32;

#[derive(Component)]
struct Chunk;

#[derive(PartialEq)]
enum BlockType {
    Air,
    Dirt,
}

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        // .add_system(bevy::window::close_on_esc)
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 12.0,          // default: 12.0
        })
        .add_startup_system(setup)
        .add_systems((chunk_handling,))
        .run();
}

macro_rules! get_chunk_position {
    ($camera_position:expr) => {{
        let chunk_position = Vec3::new(
            ($camera_position.x / CHUNK_SIZE as f32).floor() * CHUNK_SIZE as f32,
            ($camera_position.y / CHUNK_SIZE as f32).floor() * CHUNK_SIZE as f32,
            ($camera_position.z / CHUNK_SIZE as f32).floor() * CHUNK_SIZE as f32,
        );
        chunk_position
    }};
}

macro_rules! get_block_verticies {
    ($position:expr) => {{
        // Get the verticies of a block
        vec![
            [$position.x - 0.5, $position.y - 0.5, $position.z - 0.5],
            [$position.x - 0.5, $position.y - 0.5, $position.z + 0.5],
            [$position.x + 0.5, $position.y - 0.5, $position.z + 0.5],
            [$position.x + 0.5, $position.y - 0.5, $position.z - 0.5],
            [$position.x - 0.5, $position.y + 0.5, $position.z - 0.5],
            [$position.x - 0.5, $position.y + 0.5, $position.z + 0.5],
            [$position.x + 0.5, $position.y + 0.5, $position.z + 0.5],
            [$position.x + 0.5, $position.y + 0.5, $position.z - 0.5],
        ]
    }};
}

/// set up a simple 3D scene
fn setup(mut commands: Commands) {
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
}

fn chunk_handling(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera: Query<&Transform, With<FlyCam>>,
    chunks: Query<&Transform, With<Chunk>>,
) {
    // Get the camera position
    let camera_position = camera.single().translation;

    // Check if all chunks are loaded around the camera in a 8x8 chunk radius
    for x in 0..8 {
        for y in 0..8 {
            for z in 0..8 {
                // Get the position of the chunk to generate
                let chunk_position = Vec3::new(
                    (camera_position.x / CHUNK_SIZE as f32).floor() * CHUNK_SIZE as f32
                        + x as f32 * CHUNK_SIZE as f32,
                    (camera_position.y / CHUNK_SIZE as f32).floor() * CHUNK_SIZE as f32
                        + y as f32 * CHUNK_SIZE as f32,
                    (camera_position.z / CHUNK_SIZE as f32).floor() * CHUNK_SIZE as f32
                        + z as f32 * CHUNK_SIZE as f32,
                );
                // Check if the chunk is already loaded
                let mut chunk_loaded = false;
                for chunk in chunks.iter() {
                    if chunk.translation == chunk_position {
                        chunk_loaded = true;
                    }
                }
                // If the chunk is not loaded, generate it
                if !chunk_loaded {
                    generate_chunk_mesh(&mut commands, &mut meshes, &mut materials, chunk_position);
                }
            }
        }
    }
}

fn generate_chunk_mesh(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    chunk_position: Vec3,
) {
    // To generate a chunk, we need a list of verticies and a list of indicies
    let mut verticies: Vec<[f32; 3]> = Vec::new();
    // To generate a chunk we can use perlin noise
    let perlin = Perlin::new(SEED);
    // Generate a block for each position in the chunk
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                // Get the position of the block
                let position = Vec3::new(x as f32, y as f32, z as f32);
                // Get the block type
                let block_type = get_block_type(&perlin, position);
                // If the block is not air, add it to the list of verticies
                if block_type != BlockType::Air {
                    // Get the block verticies
                    let block_verticies = get_block_verticies!(position);
                    // Add the block verticies to the list of verticies
                    for vertex in block_verticies.iter() {
                        verticies.push(*vertex);
                    }
                }
            }
        }
    }
    // Remove duplicate verticies
    remove_duplicate_verticies(&mut verticies);
    // Create a mesh from the verticies
    let mesh = create_mesh(&verticies);
    // Create a chunk entity
    commands.spawn((
        Chunk,
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
            transform: Transform::from_translation(chunk_position),
            ..Default::default()
        },
    ));
    info!("Chunk generated at {:?}", chunk_position);
}

fn get_block_type(perlin: &Perlin, position: Vec3) -> BlockType {
    // Get the block type at a position
    // TODO: Add more block types
    let noise = perlin.get([position.x as f64, position.y as f64, position.z as f64]);
    if noise > 0.0 {
        BlockType::Dirt
    } else {
        BlockType::Air
    }
}

fn remove_duplicate_verticies(verticies: &mut Vec<[f32; 3]>) {
    // If there are two or more verticies at the exact same position, remove until there is only one
    for i in 0..verticies.len() {
        for j in 0..verticies.len() {
            if i != j && verticies[i] == verticies[j] {
                verticies.remove(j);
            }
        }
    }
}

// fn get_chunk_position(camera_position: Vec3) -> Vec3 {
//     // Get the position of the chunk to generate
//     let chunk_position = Vec3::new(
//         (camera_position.x / CHUNK_SIZE as f32).floor() * CHUNK_SIZE as f32,
//         (camera_position.y / CHUNK_SIZE as f32).floor() * CHUNK_SIZE as f32,
//         (camera_position.z / CHUNK_SIZE as f32).floor() * CHUNK_SIZE as f32,
//     );
//     chunk_position
// }

fn create_mesh(verticies: &Vec<[f32; 3]>) -> Mesh {
    // Create a mesh from the verticies
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verticies.to_vec());
    // Generate indices
    let mut indices: Vec<u32> = Vec::new();
    for i in 0..verticies.len() {
        indices.push(i as u32);
    }
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh
}
