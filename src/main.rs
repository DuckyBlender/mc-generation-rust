use bevy::diagnostic::SystemInformationDiagnosticsPlugin;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_atmosphere::prelude::*;
use bevy_flycam::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_debug_lines::*;
use noise::{NoiseFn, Perlin};
use std::time::Instant;

mod ivec2xz;
use ivec2xz::IVec2XZ;

const SEED: u32 = 2137;
const CHUNK_SIZE: usize = 16;
const CHUNK_HEIGHT: usize = 256;
const WORLD_SCALE: f64 = 0.1;
const NOISE_THRESHOLD: f64 = 0.3;
const RENDER_DISTANCE: i32 = 6;

#[derive(Component)]
struct ChunkMesh;

// For FPS counter
#[derive(Component)]
struct TextChanges;

#[derive(Component)]
struct ChunkBorder;

#[derive(Resource)]
struct ChunksLoaded {
    chunks: Vec<IVec2XZ>,
}

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

fn main() {
    let window = WindowPlugin {
        primary_window: Some(Window {
            title: "Bevy Voxel Demonstration".into(),
            resolution: (1280., 720.).into(),
            resizable: true,
            mode: bevy::window::WindowMode::Windowed,
            ..default()
        }),
        ..default()
    };

    App::new()
        .insert_resource(Msaa::Sample2)
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(window),
        )
        .add_plugins(NoCameraPlayerPlugin)
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 30.0,          // default: 12.0
        })
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(SystemInformationDiagnosticsPlugin)
        .add_plugins(DebugLinesPlugin::with_depth_test(true))
        .add_plugins(AtmospherePlugin)
        // Resources
        .insert_resource(ChunksLoaded { chunks: vec![] })
        // Systems
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (chunk_border, debug_keyboard, update_text, chunk_system),
        )
        .run();
}

fn setup(mut commands: Commands) {
    // Camera in 3D space.
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..default()
        },
        // FogSettings {
        //     color: Color::rgba(0.1, 0.1, 0.1, 1.0),
        //     directional_light_color: Color::rgba(1.0, 0.95, 0.75, 0.3),
        //     directional_light_exponent: 10.0,
        //     falloff: FogFalloff::from_visibility_colors(
        //         (CHUNK_SIZE * RENDER_DISTANCE as usize - CHUNK_SIZE) as f32, // distance in world units up to which objects retain visibility (>= 5% contrast)
        //         Color::BLACK,
        //         Color::BLACK,
        //     ),
        // },
        FlyCam,
        AtmosphereCamera::default(),
    ));

    // Configure a properly scaled cascade shadow map for this scene (defaults are too large, mesh units are in km)
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 0.3,
        maximum_distance: 3.0,
        ..default()
    }
    .build();

    // Sun
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .looking_at(Vec3::new(-0.15, -0.05, 0.25), Vec3::Y),
        cascade_shadow_config,
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
            bottom: Val::Px(20.0),
            left: Val::Px(12.0),
            ..default()
        }),
        TextChanges,
    ));
}

fn chunk_system(
    mut commands: Commands,
    mut chunks_loaded: ResMut<ChunksLoaded>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<&Transform, With<Camera3d>>,
    asset_server: ResMut<AssetServer>,
) {
    // Check for differences between the chunks that are loaded and the chunks that should be loaded.
    let mut chunks_to_load: Vec<IVec2XZ> = Vec::new();
    let camera_position = camera_query.single().translation;

    // Calculate the player's chunk position based on their world position.
    let player_chunk_position = IVec2XZ::new(
        (camera_position.x / CHUNK_SIZE as f32).floor() as i32,
        (camera_position.z / CHUNK_SIZE as f32).floor() as i32,
    );

    // Calculate the radius of the sphere around the player.
    let radius = RENDER_DISTANCE;

    // Loop over each chunk position within the radius.
    for x in -radius..=radius {
        for z in -radius..=radius {
            let chunk_position = player_chunk_position + IVec2XZ::new(x, z);

            // Check if the chunk is already loaded.
            if !chunks_loaded.chunks.contains(&chunk_position) {
                // Chunk is not loaded, add it to the list of chunks to load.
                chunks_to_load.push(chunk_position);
            }
        }
    }

    // Load the chunks.
    let custom_texture_handle: Handle<Image> = asset_server.load("textures/yo.png");

    for chunk_position in chunks_to_load {
        let chunk_mesh_handle: Handle<Mesh> = meshes.add(create_chunk_mesh(chunk_position));

        commands.spawn((
            PbrBundle {
                mesh: chunk_mesh_handle,
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(custom_texture_handle.clone_weak()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ChunkMesh,
        ));

        chunks_loaded.chunks.push(chunk_position);
    }
}

fn debug_keyboard(keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::F) {
        info!("F");
    }
}

/// Creates a 32x32x32 chunk mesh using 3D Perlin noise.
///
/// The mesh is created by sampling the noise function at each vertex position.
/// If the noise value is above a certain threshold, a cube is created at that position.
/// The chunk_position vec3 is the position of the chunk in the world. It is scaled down by the chunk size.
fn create_chunk_mesh(chunk_position: IVec2XZ) -> Mesh {
    // Start the timer.
    let start = Instant::now();

    // Create a new mesh.
    let mut chunk_mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    // Generate an array of Blocks, representing whether a cube should be created at that position.
    let mut chunk_blocks: [[[BlockType; CHUNK_SIZE]; CHUNK_HEIGHT]; CHUNK_SIZE] =
        [[[BlockType::Air; CHUNK_SIZE]; CHUNK_HEIGHT]; CHUNK_SIZE];

    // Create a 3D Perlin noise function with a random seed.
    let perlin = Perlin::new(SEED);

    // Loop over each block position in the chunk.
    // Remember to offset the position by the chunk position.
    #[allow(clippy::needless_range_loop)]
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                // Scale the position down by the chunk size.
                let scaled_x = x as i32 + (chunk_position.x * CHUNK_SIZE as i32);
                let scaled_y = y;
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
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                // Get the block type at the current position.
                let block_type = chunk_blocks[x][y][z];

                // If the block is Air, we don't need to create any faces.
                if block_type == BlockType::Air {
                    continue;
                }

                // Check the blocks around the current block to see if we need to create faces.

                // Check the block above.
                if y < CHUNK_HEIGHT - 1 {
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
                            IVec2XZ::new(chunk_position.x, chunk_position.z),
                            [x as f32, y as f32, z as f32],
                            BlockFace::Top,
                        );
                    }
                } else {
                    // If the block above is outside the chunk, we need to create a face.
                    create_face(
                        &mut vertices,
                        &mut indices,
                        &mut normals,
                        &mut uvs,
                        IVec2XZ::new(chunk_position.x, chunk_position.z),
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
                            IVec2XZ::new(chunk_position.x, chunk_position.z),
                            [x as f32, y as f32, z as f32],
                            BlockFace::Bottom,
                        );
                    }
                } else {
                    // If the block below is outside the chunk, we need to create a face.
                    create_face(
                        &mut vertices,
                        &mut indices,
                        &mut normals,
                        &mut uvs,
                        IVec2XZ::new(chunk_position.x, chunk_position.z),
                        [x as f32, y as f32, z as f32],
                        BlockFace::Bottom,
                    );
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
                            IVec2XZ::new(chunk_position.x, chunk_position.z),
                            [x as f32, y as f32, z as f32],
                            BlockFace::Right,
                        );
                    }
                } else {
                    // If the block to the right is outside the chunk, we need to create a face.
                    create_face(
                        &mut vertices,
                        &mut indices,
                        &mut normals,
                        &mut uvs,
                        IVec2XZ::new(chunk_position.x, chunk_position.z),
                        [x as f32, y as f32, z as f32],
                        BlockFace::Right,
                    );
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
                            IVec2XZ::new(chunk_position.x, chunk_position.z),
                            [x as f32, y as f32, z as f32],
                            BlockFace::Left,
                        );
                    }
                } else {
                    // If the block to the left is outside the chunk, we need to create a face.
                    create_face(
                        &mut vertices,
                        &mut indices,
                        &mut normals,
                        &mut uvs,
                        IVec2XZ::new(chunk_position.x, chunk_position.z),
                        [x as f32, y as f32, z as f32],
                        BlockFace::Left,
                    );
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
                            IVec2XZ::new(chunk_position.x, chunk_position.z),
                            [x as f32, y as f32, z as f32],
                            BlockFace::Front,
                        );
                    }
                } else {
                    // If the block in front is outside the chunk, we need to create a face.
                    create_face(
                        &mut vertices,
                        &mut indices,
                        &mut normals,
                        &mut uvs,
                        IVec2XZ::new(chunk_position.x, chunk_position.z),
                        [x as f32, y as f32, z as f32],
                        BlockFace::Front,
                    );
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
                            IVec2XZ::new(chunk_position.x, chunk_position.z),
                            [x as f32, y as f32, z as f32],
                            BlockFace::Back,
                        );
                    }
                } else {
                    // If the block behind is outside the chunk, we need to create a face.
                    create_face(
                        &mut vertices,
                        &mut indices,
                        &mut normals,
                        &mut uvs,
                        IVec2XZ::new(chunk_position.x, chunk_position.z),
                        [x as f32, y as f32, z as f32],
                        BlockFace::Back,
                    );
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
    chunk_position: IVec2XZ,
    position: [f32; 3],
    direction: BlockFace,
) {
    // Offset the position of the face based on the chunk position.
    let position = [
        position[0] + chunk_position.x as f32 * CHUNK_SIZE as f32,
        position[1],
        position[2] + chunk_position.z as f32 * CHUNK_SIZE as f32,
    ];

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
    // Bevy has backface culling enabled by default. This means that the vertices need to be in clockwise order. If a face is not showing up, this is probably the reason. (this took me so long)
    let face_vertices = match direction {
        BlockFace::Top => [
            [position[0], position[1] + 1.0, position[2]],
            [position[0], position[1] + 1.0, position[2] + 1.0],
            [position[0] + 1.0, position[1] + 1.0, position[2] + 1.0],
            [position[0] + 1.0, position[1] + 1.0, position[2]],
        ],
        BlockFace::Bottom => [
            [position[0], position[1], position[2] + 1.0],
            [position[0], position[1], position[2]],
            [position[0] + 1.0, position[1], position[2]],
            [position[0] + 1.0, position[1], position[2] + 1.0],
        ],

        BlockFace::Left => [
            [position[0], position[1] + 1.0, position[2]],
            [position[0], position[1], position[2]],
            [position[0], position[1], position[2] + 1.0],
            [position[0], position[1] + 1.0, position[2] + 1.0],
        ],
        BlockFace::Right => [
            [position[0] + 1.0, position[1] + 1.0, position[2] + 1.0],
            [position[0] + 1.0, position[1], position[2] + 1.0],
            [position[0] + 1.0, position[1], position[2]],
            [position[0] + 1.0, position[1] + 1.0, position[2]],
        ],
        BlockFace::Front => [
            [position[0] + 1.0, position[1] + 1.0, position[2] + 1.0],
            [position[0], position[1] + 1.0, position[2] + 1.0],
            [position[0], position[1], position[2] + 1.0],
            [position[0] + 1.0, position[1], position[2] + 1.0],
        ],
        BlockFace::Back => [
            [position[0], position[1] + 1.0, position[2]],
            [position[0] + 1.0, position[1] + 1.0, position[2]],
            [position[0] + 1.0, position[1], position[2]],
            [position[0], position[1], position[2]],
        ],
    };

    // Add the vertices and normals to the vectors.
    vertices.extend_from_slice(&face_vertices);
    normals.extend_from_slice(&[normal; 4]);

    // Add the UV coordinates to the vector.
    uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);

    // Add the indices to the vector. This is clockwise order.
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
    let y1 = CHUNK_HEIGHT as i32;
    let z1 = CHUNK_SIZE as i32;

    let x2 = x1 - CHUNK_SIZE as i32;
    let y2 = y1 - CHUNK_HEIGHT as i32;
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

    let mut cpu = 0.0;
    if let Some(cpu_diagnostic) = diagnostics.get(SystemInformationDiagnosticsPlugin::CPU_USAGE) {
        if let Some(cpu_smoothed) = cpu_diagnostic.smoothed() {
            cpu = cpu_smoothed;
        }
    }

    let mut ram = 0.0;
    if let Some(ram_diagnostic) = diagnostics.get(SystemInformationDiagnosticsPlugin::MEM_USAGE) {
        if let Some(ram_smoothed) = ram_diagnostic.smoothed() {
            ram = ram_smoothed;
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
        "FPS: {:.2}, CPU: {:.2}%, RAM: {:.2}%\nPosition: ({:.2}, {:.2}, {:.2}), Direction: ({})",
        fps, cpu, ram, camera_position.x, camera_position.y, camera_position.z, direction
    );
}
