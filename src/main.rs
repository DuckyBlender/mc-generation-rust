use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};
use bevy_flycam::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use noise::{NoiseFn, Perlin};

const CHUNK_HEIGHT: usize = 32;
const CHUNK_WIDTH: usize = 32;
const CHUNK_DEPTH: usize = 32;
const CHUNK_SIZE: usize = CHUNK_HEIGHT * CHUNK_WIDTH * CHUNK_DEPTH;

const WORLD_SCALE: f64 = 0.05;

// converting from 1d to 3d array
macro_rules! to_3d {
    ($index:expr) => {
        [
            $index % CHUNK_WIDTH,
            ($index / CHUNK_WIDTH) % CHUNK_HEIGHT,
            $index / (CHUNK_WIDTH * CHUNK_HEIGHT),
        ]
    };
}

// converting from 3d to 1d array
macro_rules! to_1d {
    ($x:expr, $y:expr, $z:expr) => {
        $x + $y * CHUNK_WIDTH + $z * CHUNK_WIDTH * CHUNK_HEIGHT
    };
}

// voxel-generation-rust

#[derive(Default, Debug, PartialEq, Clone, Copy)]
enum BlockType {
    #[default]
    Air = 0,
    Stone = 1,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(PlayerPlugin)
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 50.0,          // default: 12.0
        })
        .insert_resource(KeyBindings {
            move_ascend: KeyCode::Space,
            move_descend: KeyCode::ShiftLeft,
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    // Camera is spawned by plugin

    // Spawn sun
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        directional_light: DirectionalLight {
            color: Color::WHITE,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    // Ambient light
    ambient_light.color = Color::GRAY;

    // Spawn chunks
    for x in -10..10 {
        for y in -2..2 {
            for z in -10..10 {
                create_and_spawn_chunk(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    IVec3::new(x, y, z),
                );
            }
        }
    }
}

fn create_and_spawn_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    chunk_position: IVec3,
) {
    let now = std::time::Instant::now();
    // Create a new mesh.
    let mut chunk_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    // indices will be created later

    // For now, this function creates one chunk using 3d perlin noise
    let perlin = Perlin::new(2137);
    // 1d array for performance
    let mut chunk: [BlockType; CHUNK_SIZE] = [BlockType::Air; CHUNK_SIZE];
    for x in 0..CHUNK_WIDTH as i32 {
        for y in 0..CHUNK_HEIGHT as i32 {
            for z in 0..CHUNK_DEPTH as i32 {
                // "Globalize" the coordinates for perlin noise
                let x = x + chunk_position.x * CHUNK_WIDTH as i32;
                let y = y + chunk_position.y * CHUNK_HEIGHT as i32;
                let z = z + chunk_position.z * CHUNK_DEPTH as i32;

                let noise = perlin.get([
                    x as f64 * WORLD_SCALE,
                    y as f64 * WORLD_SCALE,
                    z as f64 * WORLD_SCALE,
                ]);

                let block = if noise > 0.0 {
                    BlockType::Stone
                } else {
                    BlockType::Air
                };

                // "Un-globalize" the coordinates for the chunk data
                let x = x - chunk_position.x * CHUNK_WIDTH as i32;
                let y = y - chunk_position.y * CHUNK_HEIGHT as i32;
                let z = z - chunk_position.z * CHUNK_DEPTH as i32;

                chunk[to_1d!(x as usize, y as usize, z as usize)] = block;
            }
        }
    }

    // Count amount of air, dirt and stone blocks
    let mut air = 0;

    let mut stone = 0;
    for block in &chunk {
        match block {
            BlockType::Air => air += 1,
            BlockType::Stone => stone += 1,
        }
    }
    info!("Air: {}, Stone: {}, Total: {}", air, stone, CHUNK_SIZE);

    // Chunk finished generating, now create verticies and indices
    for i in 0..CHUNK_SIZE {
        let block = chunk[i];
        if block == BlockType::Air {
            continue;
        }
        let [x, y, z] = to_3d!(i);

        // top (+y)
        if y == CHUNK_HEIGHT - 1 || chunk[to_1d!(x, y + 1, z)] == BlockType::Air {
            vertices.push([x as f32, y as f32 + 1.0, z as f32 + 1.0]);
            vertices.push([x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0]);
            vertices.push([x as f32 + 1.0, y as f32 + 1.0, z as f32]);
            vertices.push([x as f32, y as f32 + 1.0, z as f32]);

            // add normals
            normals.push([0.0, 1.0, 0.0]);
            normals.push([0.0, 1.0, 0.0]);
            normals.push([0.0, 1.0, 0.0]);
            normals.push([0.0, 1.0, 0.0]);
        }
        // bottom (-y)
        if y == 0 || chunk[to_1d!(x, y - 1, z)] == BlockType::Air {
            vertices.push([x as f32, y as f32, z as f32]);
            vertices.push([x as f32 + 1.0, y as f32, z as f32]);
            vertices.push([x as f32 + 1.0, y as f32, z as f32 + 1.0]);
            vertices.push([x as f32, y as f32, z as f32 + 1.0]);
            // add normals
            normals.push([0.0, -1.0, 0.0]);
            normals.push([0.0, -1.0, 0.0]);
            normals.push([0.0, -1.0, 0.0]);
            normals.push([0.0, -1.0, 0.0]);
        }
        // left (-x)
        if x == 0 || chunk[to_1d!(x - 1, y, z)] == BlockType::Air {
            vertices.push([x as f32, y as f32, z as f32 + 1.0]);
            vertices.push([x as f32, y as f32 + 1.0, z as f32 + 1.0]);
            vertices.push([x as f32, y as f32 + 1.0, z as f32]);
            vertices.push([x as f32, y as f32, z as f32]);

            // add normals
            normals.push([-1.0, 0.0, 0.0]);
            normals.push([-1.0, 0.0, 0.0]);
            normals.push([-1.0, 0.0, 0.0]);
            normals.push([-1.0, 0.0, 0.0]);
        }
        // right (+x)
        if x == CHUNK_WIDTH - 1 || chunk[to_1d!(x + 1, y, z)] == BlockType::Air {
            vertices.push([x as f32 + 1.0, y as f32, z as f32]);
            vertices.push([x as f32 + 1.0, y as f32 + 1.0, z as f32]);
            vertices.push([x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0]);
            vertices.push([x as f32 + 1.0, y as f32, z as f32 + 1.0]);
            // add normal
            normals.push([1.0, 0.0, 0.0]);
            normals.push([1.0, 0.0, 0.0]);
            normals.push([1.0, 0.0, 0.0]);
            normals.push([1.0, 0.0, 0.0]);
        }
        // front (-z)
        if z == 0 || chunk[to_1d!(x, y, z - 1)] == BlockType::Air {
            vertices.push([x as f32, y as f32 + 1.0, z as f32]);
            vertices.push([x as f32 + 1.0, y as f32 + 1.0, z as f32]);
            vertices.push([x as f32 + 1.0, y as f32, z as f32]);
            vertices.push([x as f32, y as f32, z as f32]);

            // add normal
            normals.push([0.0, 0.0, -1.0]);
            normals.push([0.0, 0.0, -1.0]);
            normals.push([0.0, 0.0, -1.0]);
            normals.push([0.0, 0.0, -1.0]);
        }
        // back (+z)
        if z == CHUNK_DEPTH - 1 || chunk[to_1d!(x, y, z + 1)] == BlockType::Air {
            vertices.push([x as f32, y as f32, z as f32 + 1.0]);
            vertices.push([x as f32 + 1.0, y as f32, z as f32 + 1.0]);
            vertices.push([x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0]);
            vertices.push([x as f32, y as f32 + 1.0, z as f32 + 1.0]);
            // add normal
            normals.push([0.0, 0.0, 1.0]);
            normals.push([0.0, 0.0, 1.0]);
            normals.push([0.0, 0.0, 1.0]);
            normals.push([0.0, 0.0, 1.0]);
        }
    }

    // Move all vertices to the correct position
    for vertex in &mut vertices {
        vertex[0] += chunk_position.x as f32 * CHUNK_WIDTH as f32;
        vertex[1] += chunk_position.y as f32 * CHUNK_HEIGHT as f32;
        vertex[2] += chunk_position.z as f32 * CHUNK_DEPTH as f32;
    }

    // Create indices
    // Calculate static array size for performance
    let mut indices: Vec<u32> = Vec::with_capacity(vertices.len() / 4 * 6);
    for i in 0..vertices.len() / 4 {
        indices.push(i as u32 * 4);
        indices.push(i as u32 * 4 + 1);
        indices.push(i as u32 * 4 + 2);
        indices.push(i as u32 * 4);
        indices.push(i as u32 * 4 + 2);
        indices.push(i as u32 * 4 + 3);
    }

    // Create uvs
    let mut uvs = Vec::new();
    for _ in 0..vertices.len() / 4 {
        uvs.push([0.0, 0.0]);
        uvs.push([1.0, 0.0]);
        uvs.push([1.0, 1.0]);
        uvs.push([0.0, 1.0]);
    }

    // Convert the vectors to VertexAttributeValues and add them to the mesh.
    chunk_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(vertices),
    );
    chunk_mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
    chunk_mesh.set_indices(Some(Indices::U32(indices)));

    let chunk_mesh_handle: Handle<Mesh> = meshes.add(chunk_mesh);

    // Spawn the chunk
    commands.spawn(PbrBundle {
        mesh: chunk_mesh_handle,
        material: materials.add(Color::rgb(0.5, 1.0, 0.0).into()),
        ..Default::default()
    });

    let elapsed = now.elapsed().as_nanos();

    info!("Chunk generated in {} ns", elapsed)
}
