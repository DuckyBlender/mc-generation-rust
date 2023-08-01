use std::time::Instant;

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_flycam::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_debug_lines::*;
use noise::{NoiseFn, Perlin};

const SEED: u32 = 2137;
const CHUNK_SIZE: usize = 16;
const WORLD_SCALE: f64 = 0.1;
const NOISE_THRESHOLD: f64 = 0.3;

// Define a "marker" component to mark the custom mesh. Marker components are often used in Bevy for
// filtering entities in queries with With, they're usually not queried directly since they don't contain information within them.
#[derive(Component)]
struct CustomUV;

// For FPS counter
#[derive(Component)]
struct TextChanges;

#[derive(Component)]
struct ChunkBorder;

#[derive(Component)]
struct Chunk;

#[derive(PartialEq, Debug, Copy, Clone)]
enum BlockType {
    Air,
    Dirt,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum BlockFace {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

#[derive(Clone, Debug)]
struct Block {
    block_type: BlockType,
    air_intersections: Vec<BlockFace>,
    block_position: IVec3,
}

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(NoCameraPlayerPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(DebugLinesPlugin::default())
        // Systems
        .add_systems(Startup, setup)
        .add_systems(Update, chunk_border)
        // .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Update, update_text)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Import the custom texture.
    let custom_texture_handle: Handle<Image> = asset_server.load("textures/yo.png");

    // Spawn chunks
    for x in 0..10 {
        for y in 0..2 {
            for z in 0..10 {
                let chunk_mesh_handle: Handle<Mesh> =
                    meshes.add(create_chunk_mesh(IVec3::new(x, y, z)));

                commands.spawn((
                    PbrBundle {
                        mesh: chunk_mesh_handle,
                        material: materials.add(StandardMaterial {
                            base_color_texture: Some(custom_texture_handle.clone()),
                            ..default()
                        }),
                        ..default()
                    },
                    CustomUV,
                ));
            }
        }
    }

    // Transform for the camera and lighting, looking at (0,0,0) (the position of the mesh).
    let camera_and_light_transform =
        Transform::from_xyz(20., 20., 20.).looking_at(Vec3::ZERO, Vec3::Y);

    // Camera in 3D space.
    commands.spawn((
        Camera3dBundle {
            transform: camera_and_light_transform,
            ..default()
        },
        FlyCam,
    ));

    // Light up the scene.
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1000.0,
            range: 500.0,
            ..default()
        },
        transform: camera_and_light_transform,
        ..default()
    });

    // Text to display FPS
    commands.spawn((
        TextBundle::from_section(
            "".to_string(),
            TextStyle {
                font_size: 20.0,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
        TextChanges,
    ));
}

fn create_cube_mesh() -> Mesh {
    let mut cube_mesh = Mesh::new(PrimitiveTopology::TriangleList);

    #[rustfmt::skip]
    cube_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        // Each array is an [x, y, z] coordinate in local space.
        // Meshes always rotate around their local [0, 0, 0] when a rotation is applied to their Transform.
        // By centering our mesh around the origin, rotating the mesh preserves its center of mass.
        vec![
            // top (facing towards +y)
            [-0.5, 0.5, -0.5], // vertex with index 0
            [0.5, 0.5, -0.5], // vertex with index 1
            [0.5, 0.5, 0.5], // etc. until 23
            [-0.5, 0.5, 0.5],
            // bottom   (-y)
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [-0.5, -0.5, 0.5],
            // right    (+x)
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5], // This vertex is at the same position as vertex with index 2, but they'll have different UV and normal
            [0.5, 0.5, -0.5],
            // left     (-x)
            [-0.5, -0.5, -0.5],
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [-0.5, 0.5, -0.5],
            // back     (+z)
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [0.5, 0.5, 0.5],
            [0.5, -0.5, 0.5],
            // forward  (-z)
            [-0.5, -0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [0.5, 0.5, -0.5],
            [0.5, -0.5, -0.5],
        ],
    );

    // Set-up UV coordinates
    // Note: (0.0, 0.0) = Top-Left in UV mapping, (1.0, 1.0) = Bottom-Right in UV mapping
    #[rustfmt::skip]
    cube_mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![
            // Assigning the UV coords for the top side.
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // Assigning the UV coords for the bottom side.
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // Assigning the UV coords for the right side.
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // Assigning the UV coords for the left side. 
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // Assigning the UV coords for the back side.
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
            // Assigning the UV coords for the forward side.
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        ],
    );

    // For meshes with flat shading, normals are orthogonal (pointing out) from the direction of
    // the surface.
    // Normals are required for correct lighting calculations.
    // Each array represents a normalized vector, which length should be equal to 1.0.
    #[rustfmt::skip]
    cube_mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![
            // Normals for the top side (towards +y)
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            // Normals for the bottom side (towards -y)
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            // Normals for the right side (towards +x)
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            // Normals for the left side (towards -x)
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            // Normals for the back side (towards +z)
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            // Normals for the forward side (towards -z)
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
        ],
    );

    // Create the triangles out of the 24 vertices we created.
    // To construct a square, we need 2 triangles, therefore 12 triangles in total.
    // To construct a triangle, we need the indices of its 3 defined vertices, adding them one
    // by one, in a counter-clockwise order (relative to the position of the viewer, the order
    // should appear counter-clockwise from the front of the triangle, in this case from outside the cube).
    // Read more about how to correctly build a mesh manually in the Bevy documentation of a Mesh,
    // further examples and the implementation of the built-in shapes.
    #[rustfmt::skip]
    cube_mesh.set_indices(Some(Indices::U32(vec![
        0,3,1 , 1,3,2, // triangles making up the top (+y) facing side.
        4,5,7 , 5,6,7, // bottom (-y) 
        8,11,9 , 9,11,10, // right (+x)
        12,13,15 , 13,14,15, // left (-x)
        16,19,17 , 17,19,18, // back (+z)
        20,21,23 , 21,22,23, // forward (-z)
    ])));

    cube_mesh
}

/// Creates a 32x32x32 chunk mesh using 3D Perlin noise.
///
/// The mesh is created by sampling the noise function at each vertex position.
/// If the noise value is above a certain threshold, a cube is created at that position.
/// The chunk_position vec3 is the position of the chunk in the world. It is scaled down by the chunk size.
fn create_chunk_mesh(chunk_position: IVec3) -> Mesh {
    // Start the timer.
    let start = Instant::now();

    // Create a new mesh.
    let mut chunk_mesh = Mesh::new(PrimitiveTopology::TriangleList);

    // Generate a vector of Blocks, representing whether a cube should be created at that position.
    let mut chunk_data: Vec<Block> = Vec::new();

    // Create a 3D Perlin noise function with a random seed.
    let perlin = Perlin::new(SEED);

    // Loop over each block position in the chunk.
    // Remember to offset the position by the chunk position.
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                // Scale the position down by the chunk size.
                let scaled_x = x as i32 + (chunk_position.x * CHUNK_SIZE as i32);
                let scaled_y = y as i32 + (chunk_position.y * CHUNK_SIZE as i32);
                let scaled_z = z as i32 + (chunk_position.z * CHUNK_SIZE as i32);
                // info!("Scaled position: {}, {}, {}", scaled_x, scaled_y, scaled_z);

                // Sample the noise function at the scaled position.
                // The perlin noise needs a float value, so we need to cast the scaled position to a float.
                let noise_value = perlin.get([
                    scaled_x as f64 * WORLD_SCALE,
                    scaled_y as f64 * WORLD_SCALE,
                    scaled_z as f64 * WORLD_SCALE,
                ]);

                // If the noise value is above the threshold, create a cube at that position.
                if noise_value > NOISE_THRESHOLD {
                    chunk_data.push(Block {
                        block_type: BlockType::Dirt,
                        air_intersections: Vec::new(),
                        block_position: IVec3::new(scaled_x, scaled_y, scaled_z),
                    });
                } else {
                    chunk_data.push(Block {
                        block_type: BlockType::Air,
                        air_intersections: Vec::new(),
                        block_position: IVec3::new(scaled_x, scaled_y, scaled_z),
                    });
                }
            }
        }
    }
    // Now that the chunk data is generated, check the neighbouring blocks to see if we need to create faces.
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                // Scale the position down by the chunk size.

                // Get the index of the current block.
                let index = x + (y * CHUNK_SIZE) + (z * CHUNK_SIZE * CHUNK_SIZE);

                // Get the block at the current index.
                let current_block = &chunk_data[index];
                if current_block.block_type == BlockType::Air {
                    continue;
                }

                let above_block_id = get_neighbor_index(&index, &BlockFace::Top, &chunk_position);
                let below_block_id =
                    get_neighbor_index(&index, &BlockFace::Bottom, &chunk_position);
                let left_block_id = get_neighbor_index(&index, &BlockFace::Left, &chunk_position);
                let right_block_id = get_neighbor_index(&index, &BlockFace::Right, &chunk_position);
                let front_block_id = get_neighbor_index(&index, &BlockFace::Front, &chunk_position);
                let back_block_id = get_neighbor_index(&index, &BlockFace::Back, &chunk_position);

                // Unwrap all the block ids.
                if let Some(above_block_id) = above_block_id {
                    let above_block = &chunk_data[above_block_id];
                    if above_block.block_type == BlockType::Air {
                        // Update the air_intersections of the current block.
                        chunk_data[index].air_intersections.push(BlockFace::Top);
                    }
                }

                if let Some(below_block_id) = below_block_id {
                    let below_block = &chunk_data[below_block_id];
                    if below_block.block_type == BlockType::Air {
                        // Update the air_intersections of the current block.
                        chunk_data[index].air_intersections.push(BlockFace::Bottom);
                    }
                }

                if let Some(left_block_id) = left_block_id {
                    let left_block = &chunk_data[left_block_id];
                    if left_block.block_type == BlockType::Air {
                        // Update the air_intersections of the current block.
                        chunk_data[index].air_intersections.push(BlockFace::Left);
                    }
                }

                if let Some(right_block_id) = right_block_id {
                    let right_block = &chunk_data[right_block_id];
                    if right_block.block_type == BlockType::Air {
                        // Update the air_intersections of the current block.
                        chunk_data[index].air_intersections.push(BlockFace::Right);
                    }
                }

                if let Some(front_block_id) = front_block_id {
                    let front_block = &chunk_data[front_block_id];
                    if front_block.block_type == BlockType::Air {
                        // Update the air_intersections of the current block.
                        chunk_data[index].air_intersections.push(BlockFace::Front);
                    }
                }

                if let Some(back_block_id) = back_block_id {
                    let back_block = &chunk_data[back_block_id];
                    if back_block.block_type == BlockType::Air {
                        // Update the air_intersections of the current block.
                        chunk_data[index].air_intersections.push(BlockFace::Back);
                    }
                }
            }
        }
    }

    // We now have all the information we need to create the mesh.
    // We need to create a face for each block that intersects with air.
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    for block in &chunk_data {
        if block.block_type == BlockType::Air {
            continue;
        }

        // Get the position of the current block.
        let block_position = block.block_position;

        // Get the air intersections of the current block.
        let air_intersections = &block.air_intersections;

        // Get the index of the current block.
        let mut index = vertices.len() as u32;

        // Create a face for each air intersection.
        for air_intersection in air_intersections {
            match air_intersection {
                BlockFace::Top => {
                    // Create the vertices for the top face.
                    vertices.push([
                        block_position.x as f32,
                        block_position.y as f32 + 1.0,
                        block_position.z as f32,
                    ]);
                    vertices.push([
                        block_position.x as f32,
                        block_position.y as f32 + 1.0,
                        block_position.z as f32 + 1.0,
                    ]);
                    vertices.push([
                        block_position.x as f32 + 1.0,
                        block_position.y as f32 + 1.0,
                        block_position.z as f32 + 1.0,
                    ]);
                    vertices.push([
                        block_position.x as f32 + 1.0,
                        block_position.y as f32 + 1.0,
                        block_position.z as f32,
                    ]);
                    // Create the indices for the top face.
                    indices.push(index);
                    indices.push(index + 1);
                    indices.push(index + 2);
                    indices.push(index);
                    indices.push(index + 2);
                    indices.push(index + 3);

                    // Create the normals for the top face.
                    normals.push([0.0, 1.0, 0.0]);
                    normals.push([0.0, 1.0, 0.0]);
                    normals.push([0.0, 1.0, 0.0]);
                    normals.push([0.0, 1.0, 0.0]);

                    // Create the uvs for the top face.
                    uvs.push([0.0, 0.0]);
                    uvs.push([1.0, 0.0]);
                    uvs.push([1.0, 1.0]);
                    uvs.push([0.0, 1.0]);

                    // Update the index for the next face.
                    index += 4;
                }
                BlockFace::Bottom => {
                    // Create the vertices for the bottom face.
                    vertices.push([
                        block_position.x as f32,
                        block_position.y as f32,
                        block_position.z as f32 + 1.0,
                    ]);
                    vertices.push([
                        block_position.x as f32 + 1.0,
                        block_position.y as f32,
                        block_position.z as f32 + 1.0,
                    ]);
                    vertices.push([
                        block_position.x as f32 + 1.0,
                        block_position.y as f32,
                        block_position.z as f32,
                    ]);
                    vertices.push([
                        block_position.x as f32,
                        block_position.y as f32,
                        block_position.z as f32,
                    ]);

                    // Create the indices for the bottom face.
                    indices.push(index);
                    indices.push(index + 2);
                    indices.push(index + 1);
                    indices.push(index);
                    indices.push(index + 3);
                    indices.push(index + 2);

                    // Create the normals for the bottom face.
                    normals.push([0.0, -1.0, 0.0]);
                    normals.push([0.0, -1.0, 0.0]);
                    normals.push([0.0, -1.0, 0.0]);
                    normals.push([0.0, -1.0, 0.0]);

                    // Create the uvs for the bottom face.
                    uvs.push([0.0, 0.0]);
                    uvs.push([1.0, 0.0]);
                    uvs.push([1.0, 1.0]);
                    uvs.push([0.0, 1.0]);

                    // Update the index.
                    index += 4;
                }

                BlockFace::Left => {
                    // Create the vertices for the left face.
                    vertices.push([
                        block_position.x as f32,
                        block_position.y as f32,
                        block_position.z as f32 + 1.0,
                    ]);
                    vertices.push([
                        block_position.x as f32,
                        block_position.y as f32 + 1.0,
                        block_position.z as f32 + 1.0,
                    ]);
                    vertices.push([
                        block_position.x as f32,
                        block_position.y as f32 + 1.0,
                        block_position.z as f32,
                    ]);
                    vertices.push([
                        block_position.x as f32,
                        block_position.y as f32,
                        block_position.z as f32,
                    ]);

                    // Create the indices for the left face.
                    indices.push(index);
                    indices.push(index + 1);
                    indices.push(index + 2);
                    indices.push(index);
                    indices.push(index + 2);
                    indices.push(index + 3);

                    // Create the normals for the left face.
                    normals.push([-1.0, 0.0, 0.0]);
                    normals.push([-1.0, 0.0, 0.0]);
                    normals.push([-1.0, 0.0, 0.0]);
                    normals.push([-1.0, 0.0, 0.0]);

                    // Create the uvs for the left face.
                    uvs.push([0.0, 0.0]);
                    uvs.push([1.0, 0.0]);
                    uvs.push([1.0, 1.0]);
                    uvs.push([0.0, 1.0]);

                    // Update the index.
                    index += 4;
                }
                BlockFace::Right => {
                    // Create the vertices for the right face.
                    vertices.push([
                        block_position.x as f32 + 1.0,
                        block_position.y as f32,
                        block_position.z as f32,
                    ]);
                    vertices.push([
                        block_position.x as f32 + 1.0,
                        block_position.y as f32 + 1.0,
                        block_position.z as f32,
                    ]);
                    vertices.push([
                        block_position.x as f32 + 1.0,
                        block_position.y as f32 + 1.0,
                        block_position.z as f32 + 1.0,
                    ]);
                    vertices.push([
                        block_position.x as f32 + 1.0,
                        block_position.y as f32,
                        block_position.z as f32 + 1.0,
                    ]);

                    // Create the indices for the right face.
                    indices.push(index);
                    indices.push(index + 2);
                    indices.push(index + 1);
                    indices.push(index);
                    indices.push(index + 3);
                    indices.push(index + 2);

                    // Create the normals for the right face.
                    normals.push([1.0, 0.0, 0.0]);
                    normals.push([1.0, 0.0, 0.0]);
                    normals.push([1.0, 0.0, 0.0]);
                    normals.push([1.0, 0.0, 0.0]);

                    // Create the uvs for the right face.
                    uvs.push([0.0, 0.0]);
                    uvs.push([1.0, 0.0]);
                    uvs.push([1.0, 1.0]);
                    uvs.push([0.0, 1.0]);

                    // Update the index.
                    index += 4;
                }
                BlockFace::Front => {
                    // Create the vertices for the front face.
                    vertices.push([
                        block_position.x as f32,
                        block_position.y as f32,
                        block_position.z as f32,
                    ]);
                    vertices.push([
                        block_position.x as f32,
                        block_position.y as f32 + 1.0,
                        block_position.z as f32,
                    ]);
                    vertices.push([
                        block_position.x as f32 + 1.0,
                        block_position.y as f32 + 1.0,
                        block_position.z as f32,
                    ]);
                    vertices.push([
                        block_position.x as f32 + 1.0,
                        block_position.y as f32,
                        block_position.z as f32,
                    ]);

                    // Create the indices for the front face.
                    indices.push(index);
                    indices.push(index + 1);
                    indices.push(index + 2);
                    indices.push(index);
                    indices.push(index + 2);
                    indices.push(index + 3);

                    // Create the normals for the front face.
                    normals.push([0.0, 0.0, -1.0]);
                    normals.push([0.0, 0.0, -1.0]);
                    normals.push([0.0, 0.0, -1.0]);
                    normals.push([0.0, 0.0, -1.0]);

                    // Create the uvs for the front face.
                    uvs.push([0.0, 0.0]);
                    uvs.push([1.0, 0.0]);
                    uvs.push([1.0, 1.0]);
                    uvs.push([0.0, 1.0]);

                    // Update the index.
                    index += 4;
                }
                BlockFace::Back => {
                    // Create the vertices for the back face.
                    vertices.push([
                        block_position.x as f32,
                        block_position.y as f32 + 1.0,
                        block_position.z as f32 + 1.0,
                    ]);
                    vertices.push([
                        block_position.x as f32,
                        block_position.y as f32,
                        block_position.z as f32 + 1.0,
                    ]);
                    vertices.push([
                        block_position.x as f32 + 1.0,
                        block_position.y as f32,
                        block_position.z as f32 + 1.0,
                    ]);
                    vertices.push([
                        block_position.x as f32 + 1.0,
                        block_position.y as f32 + 1.0,
                        block_position.z as f32 + 1.0,
                    ]);

                    // Create the indices for the back face.
                    indices.push(index);
                    indices.push(index + 2);
                    indices.push(index + 1);
                    indices.push(index);
                    indices.push(index + 3);
                    indices.push(index + 2);

                    // Create the normals for the back face.
                    normals.push([0.0, 0.0, 1.0]);
                    normals.push([0.0, 0.0, 1.0]);
                    normals.push([0.0, 0.0, 1.0]);
                    normals.push([0.0, 0.0, 1.0]);

                    // Create the uvs for the back face.
                    uvs.push([0.0, 0.0]);
                    uvs.push([1.0, 0.0]);
                    uvs.push([1.0, 1.0]);
                    uvs.push([0.0, 1.0]);

                    // Update the index.
                    index += 4;
                }
            }
        }
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

    // Stop the timer
    let elapsed = start.elapsed();
    info!("Chunk generation took: {:?}", elapsed);

    // Count the amount of Air and Dirt blocks.
    let mut air_blocks = 0;
    let mut dirt_blocks = 0;
    for block in &chunk_data {
        match block.block_type {
            BlockType::Air => air_blocks += 1,
            BlockType::Dirt => dirt_blocks += 1,
        }
    }
    info!("Air blocks: {}", air_blocks);
    info!("Dirt blocks: {}", dirt_blocks);
    // Count average intersections.
    let mut total_intersections = 0;
    for block in &chunk_data {
        total_intersections += block.air_intersections.len();
    }
    info!(
        "Average intersections: {}",
        total_intersections as f64 / chunk_data.len() as f64
    );
    // Count total intersections for all sides
    let (mut top, mut bottom, mut left, mut right, mut front, mut back) = (0, 0, 0, 0, 0, 0);
    for block in &chunk_data {
        for intersection in &block.air_intersections {
            match intersection {
                BlockFace::Top => top += 1,
                BlockFace::Bottom => bottom += 1,
                BlockFace::Left => left += 1,
                BlockFace::Right => right += 1,
                BlockFace::Front => front += 1,
                BlockFace::Back => back += 1,
            }
        }
    }
    info!(
        "Air Intersections: T: {}, B: {}, L: {}, R: {}, F: {}, B: {}",
        top, bottom, left, right, front, back
    );

    chunk_mesh
}

/// Decodes 1D index to 3D coordinates.
///
/// Reverses this formula: `index = x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE`
fn get_xyz(index: &usize, chunk_position: &IVec3) -> IVec3 {
    let x = index % CHUNK_SIZE;
    let y = (index / CHUNK_SIZE) % CHUNK_SIZE;
    let z = (index / CHUNK_SIZE / CHUNK_SIZE) % CHUNK_SIZE;
    IVec3::new(x as i32, y as i32, z as i32) + *chunk_position
}

// #[derive(PartialEq, Debug, Copy, Clone)]
// enum BlockType {
//     Air,
//     Dirt,
// }

// #[derive(Clone, Copy, Debug, PartialEq)]
// enum BlockFace {
//     Top,
//     Bottom,
//     Left,
//     Right,
//     Front,
//     Back,
// }

// #[derive(Clone, Debug)]
// struct Block {
//     block_type: BlockType,
//     air_intersections: Vec<BlockFace>,
//     block_position: IVec3,
// }

/// Get the 1D index of a neighbor block.
///
/// Returns None if the neighbor is outside of the chunk.
fn get_neighbor_index(index: &usize, face: &BlockFace, chunk_position: &IVec3) -> Option<usize> {
    let xyz = get_xyz(index, chunk_position);
    let mut neighbor_xyz = xyz;
    match face {
        BlockFace::Top => neighbor_xyz.y += 1,
        BlockFace::Bottom => neighbor_xyz.y -= 1,
        BlockFace::Left => neighbor_xyz.x -= 1,
        BlockFace::Right => neighbor_xyz.x += 1,
        BlockFace::Front => neighbor_xyz.z += 1,
        BlockFace::Back => neighbor_xyz.z -= 1,
    }
    // Check if the neighbor is outside of the chunk.
    if neighbor_xyz.x < 0
        || neighbor_xyz.x >= CHUNK_SIZE as i32
        || neighbor_xyz.y < 0
        || neighbor_xyz.y >= CHUNK_SIZE as i32
        || neighbor_xyz.z < 0
        || neighbor_xyz.z >= CHUNK_SIZE as i32
    {
        return None;
    }
    // Convert the 3D coordinates to a 1D index.
    let neighbor_index = neighbor_xyz.x as usize
        + neighbor_xyz.y as usize * CHUNK_SIZE
        + neighbor_xyz.z as usize * CHUNK_SIZE * CHUNK_SIZE;
    Some(neighbor_index)
}

fn chunk_border(
    // chunk_position: IVec3,
    // chunk_borders: Query<&ChunkBorder>,
    mut lines: ResMut<DebugLines>,
) {
    // Draw a "box" around the selected chunk.
    // Determine the coordinates
    let x1 = CHUNK_SIZE as i32;
    let y1 = CHUNK_SIZE as i32;
    let z1 = CHUNK_SIZE as i32;

    let x2 = x1 - CHUNK_SIZE as i32;
    let y2 = y1 - CHUNK_SIZE as i32;
    let z2 = z1 - CHUNK_SIZE as i32;

    // Draw the lines.
    lines.line_colored(
        [x1 as f32, y1 as f32, z1 as f32].into(),
        [x2 as f32, y1 as f32, z1 as f32].into(),
        1.0,
        Color::rgb(1.0, 0.0, 0.0),
    );
    lines.line_colored(
        [x1 as f32, y1 as f32, z1 as f32].into(),
        [x1 as f32, y2 as f32, z1 as f32].into(),
        1.0,
        Color::rgb(1.0, 0.0, 0.0),
    );
    lines.line_colored(
        [x1 as f32, y1 as f32, z1 as f32].into(),
        [x1 as f32, y1 as f32, z2 as f32].into(),
        1.0,
        Color::rgb(1.0, 0.0, 0.0),
    );
    lines.line_colored(
        [x2 as f32, y1 as f32, z1 as f32].into(),
        [x2 as f32, y2 as f32, z1 as f32].into(),
        1.0,
        Color::rgb(1.0, 0.0, 0.0),
    );
    lines.line_colored(
        [x2 as f32, y1 as f32, z1 as f32].into(),
        [x2 as f32, y1 as f32, z2 as f32].into(),
        1.0,
        Color::rgb(1.0, 0.0, 0.0),
    );
    lines.line_colored(
        [x1 as f32, y2 as f32, z1 as f32].into(),
        [x2 as f32, y2 as f32, z1 as f32].into(),
        1.0,
        Color::rgb(1.0, 0.0, 0.0),
    );
    lines.line_colored(
        [x1 as f32, y2 as f32, z1 as f32].into(),
        [x1 as f32, y2 as f32, z2 as f32].into(),
        1.0,
        Color::rgb(1.0, 0.0, 0.0),
    );
    lines.line_colored(
        [x1 as f32, y1 as f32, z2 as f32].into(),
        [x2 as f32, y1 as f32, z2 as f32].into(),
        1.0,
        Color::rgb(1.0, 0.0, 0.0),
    );
    lines.line_colored(
        [x1 as f32, y1 as f32, z2 as f32].into(),
        [x1 as f32, y2 as f32, z2 as f32].into(),
        1.0,
        Color::rgb(1.0, 0.0, 0.0),
    );
    lines.line_colored(
        [x2 as f32, y1 as f32, z2 as f32].into(),
        [x2 as f32, y2 as f32, z2 as f32].into(),
        1.0,
        Color::rgb(1.0, 0.0, 0.0),
    );
    lines.line_colored(
        [x1 as f32, y2 as f32, z2 as f32].into(),
        [x2 as f32, y2 as f32, z2 as f32].into(),
        1.0,
        Color::rgb(1.0, 0.0, 0.0),
    );
    lines.line_colored(
        [x2 as f32, y2 as f32, z1 as f32].into(),
        [x2 as f32, y2 as f32, z2 as f32].into(),
        1.0,
        Color::rgb(1.0, 0.0, 0.0),
    );
}

/// Updates the UI text.
///
/// Information about the FPS, coordinates and direction is displayed.
fn update_text(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<TextChanges>>,
    camera_query: Query<&Transform, With<Camera>>,
) {
    // Update the FPS counter.
    let mut fps_text = query.single_mut();

    let mut fps = 0.0;
    if let Some(fps_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps_smoothed) = fps_diagnostic.smoothed() {
            fps = fps_smoothed;
        }
    }

    // Update the coordinates and direction.
    let camera_transform = camera_query.single();
    let camera_position = camera_transform.translation;
    // Determine if the camera is looking towards +X, -X, +Z or -Z.
    // First determine the angle between the camera's forward vector and the +X axis.
    let forward = camera_transform.forward();
    let angle = forward.angle_between(Vec3::X);
    // Then determine the angle between the camera's forward vector and the -X axis.
    let angle2 = forward.angle_between(-Vec3::X);
    // Then determine the angle between the camera's forward vector and the +Z axis.
    let angle3 = forward.angle_between(Vec3::Z);
    // Then determine the angle between the camera's forward vector and the -Z axis.
    let angle4 = forward.angle_between(-Vec3::Z);
    // The direction is the one with the smallest angle.
    let direction = if angle < angle2 && angle < angle3 && angle < angle4 {
        "+X"
    } else if angle2 < angle && angle2 < angle3 && angle2 < angle4 {
        "-X"
    } else if angle3 < angle && angle3 < angle2 && angle3 < angle4 {
        "+Z"
    } else {
        "-Z"
    };

    fps_text.sections[0].value = format!(
        "FPS: {:.2}, Position: ({:.2}, {:.2}, {:.2}), Direction: ({})",
        fps, camera_position.x, camera_position.y, camera_position.z, direction
    );
}
