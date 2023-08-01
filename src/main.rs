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
        .add_plugins(DebugLinesPlugin::with_depth_test(true))
        // Systems
        .add_systems(Startup, setup)
        .add_systems(Update, chunk_border)
        .add_systems(Update, debug_keyboard)
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
    // For now only one chunk
    for x in 0..1 {
        for y in 0..1 {
            for z in 0..1 {
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

fn debug_keyboard(keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::F) {
        info!("F");
    }
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

    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    // Generate an array of Blocks, representing whether a cube should be created at that position.
    let mut chunk_blocks: [[[BlockType; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE] =
        [[[BlockType::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

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
                    chunk_blocks[x][y][z] = BlockType::Dirt;
                }
            }
        }
    }
    // From now on, we don't need the chunk position anymore, so we can just use the local block position.
    // Now that the chunk data is generated, check the neighbouring blocks to see if we need to create faces.
    // Loop over each block position in the chunk.
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                // Get the block type at the current position.
                let block_type = chunk_blocks[x][y][z];

                // If the block is Air, we don't need to create any faces.
                if block_type == BlockType::Air {
                    continue;
                }

                // Check the blocks around the current block to see if we need to create faces.

                // Check the block above.
                if y < CHUNK_SIZE - 1 {
                    // Get the block type of the block above.
                    let block_above = chunk_blocks[x][y + 1][z];
                    // If the block above is Air, we need to create a face.
                    if block_above == BlockType::Air {
                        // Create the face.
                        create_face(
                            &mut vertices,
                            &mut indices,
                            &mut normals,
                            &mut uvs,
                            [x as f32, y as f32, z as f32],
                            BlockFace::Top,
                        );
                    }

                    // Check the block below.
                    if y > 0 {
                        // Get the block type of the block below.
                        let block_below = chunk_blocks[x][y - 1][z];
                        // If the block below is Air, we need to create a face.
                        if block_below == BlockType::Air {
                            // Create the face.
                            create_face(
                                &mut vertices,
                                &mut indices,
                                &mut normals,
                                &mut uvs,
                                [x as f32, y as f32, z as f32],
                                BlockFace::Bottom,
                            );
                        }
                    }

                    // Check the block to the right.
                    if x < CHUNK_SIZE - 1 {
                        // Get the block type of the block to the right.
                        let block_right = chunk_blocks[x + 1][y][z];
                        // If the block to the right is Air, we need to create a face.
                        if block_right == BlockType::Air {
                            // Create the face.
                            create_face(
                                &mut vertices,
                                &mut indices,
                                &mut normals,
                                &mut uvs,
                                [x as f32, y as f32, z as f32],
                                BlockFace::Right,
                            );
                        }
                    }

                    // Check the block to the left.
                    if x > 0 {
                        // Get the block type of the block to the left.
                        let block_left = chunk_blocks[x - 1][y][z];
                        // If the block to the left is Air, we need to create a face.
                        if block_left == BlockType::Air {
                            // Create the face.
                            create_face(
                                &mut vertices,
                                &mut indices,
                                &mut normals,
                                &mut uvs,
                                [x as f32, y as f32, z as f32],
                                BlockFace::Left,
                            );
                        }
                    }

                    // Check the block in front.
                    if z < CHUNK_SIZE - 1 {
                        // Get the block type of the block in front.
                        let block_front = chunk_blocks[x][y][z + 1];
                        // If the block in front is Air, we need to create a face.
                        if block_front == BlockType::Air {
                            // Create the face.
                            create_face(
                                &mut vertices,
                                &mut indices,
                                &mut normals,
                                &mut uvs,
                                [x as f32, y as f32, z as f32],
                                BlockFace::Front,
                            );
                        }
                    }

                    // Check the block behind.
                    if z > 0 {
                        // Get the block type of the block behind.
                        let block_behind = chunk_blocks[x][y][z - 1];
                        // If the block behind is Air, we need to create a face.
                        if block_behind == BlockType::Air {
                            // Create the face.
                            create_face(
                                &mut vertices,
                                &mut indices,
                                &mut normals,
                                &mut uvs,
                                [x as f32, y as f32, z as f32],
                                BlockFace::Back,
                            );
                        }
                    }
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

    chunk_mesh
}

/// Creates a face on a block.
///
/// The `position` is the position of the block in the chunk.
/// The `direction` is the direction of the face on the block.
fn create_face(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    position: [f32; 3],
    direction: BlockFace,
) {
    // The index of the first vertex of this face.
    let first_vertex = vertices.len() as u32;

    // The normal of the face.
    let normal = match direction {
        BlockFace::Top => [0.0, 1.0, 0.0],
        BlockFace::Bottom => [0.0, -1.0, 0.0],
        BlockFace::Left => [-1.0, 0.0, 0.0],
        BlockFace::Right => [1.0, 0.0, 0.0],
        BlockFace::Front => [0.0, 0.0, 1.0],
        BlockFace::Back => [0.0, 0.0, -1.0],
    };

    // The vertices of the face.
    let face_vertices = match direction {
        BlockFace::Top => [
            [position[0], position[1] + 1.0, position[2]],
            [position[0] + 1.0, position[1] + 1.0, position[2]],
            [position[0] + 1.0, position[1] + 1.0, position[2] + 1.0],
            [position[0], position[1] + 1.0, position[2] + 1.0],
        ],
        BlockFace::Bottom => [
            [position[0], position[1], position[2]],
            [position[0] + 1.0, position[1], position[2]],
            [position[0] + 1.0, position[1], position[2] + 1.0],
            [position[0], position[1], position[2] + 1.0],
        ],
        BlockFace::Left => [
            [position[0], position[1], position[2]],
            [position[0], position[1], position[2] + 1.0],
            [position[0], position[1] + 1.0, position[2] + 1.0],
            [position[0], position[1] + 1.0, position[2]],
        ],
        BlockFace::Right => [
            [position[0] + 1.0, position[1], position[2]],
            [position[0] + 1.0, position[1], position[2] + 1.0],
            [position[0] + 1.0, position[1] + 1.0, position[2] + 1.0],
            [position[0] + 1.0, position[1] + 1.0, position[2]],
        ],
        BlockFace::Front => [
            [position[0], position[1], position[2] + 1.0],
            [position[0] + 1.0, position[1], position[2] + 1.0],
            [position[0] + 1.0, position[1] + 1.0, position[2] + 1.0],
            [position[0], position[1] + 1.0, position[2] + 1.0],
        ],
        BlockFace::Back => [
            [position[0], position[1], position[2]],
            [position[0] + 1.0, position[1], position[2]],
            [position[0] + 1.0, position[1] + 1.0, position[2]],
            [position[0], position[1] + 1.0, position[2]],
        ],
    };

    // Add the vertices and normals to the vectors.
    vertices.extend_from_slice(&face_vertices);
    normals.extend_from_slice(&[normal; 4]);

    // Add the UV coordinates to the vector.
    uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);

    // Add the indices to the vector.
    indices.extend_from_slice(&[
        first_vertex,
        first_vertex + 1,
        first_vertex + 2,
        first_vertex,
        first_vertex + 2,
        first_vertex + 3,
    ]);
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
