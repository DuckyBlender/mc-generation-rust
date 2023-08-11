use bevy::tasks::AsyncComputeTaskPool;
use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};
use bevy_rapier3d::prelude::*;
// use color_eyre::owo_colors::colors::xterm::BlueStone;
use futures_lite::future;
use noise::{NoiseFn, Perlin};
use std::collections::HashSet;
use std::time::Instant;

use super::common::*;

/// Creates a 16x256x16 chunk mesh using a combination of 3D and 2D Perlin noise.
fn create_chunk_mesh(chunk_position: IVec2XZ, game_texture: GameTextureAtlas) -> Mesh {
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

    // Create a 3D Perlin noise function with a random seed for the cave and surface generation
    let perlin = Perlin::new(SEED);

    // Loop over each block position in the chunk.
    // Remember to offset the position by the chunk position.
    #[allow(clippy::needless_range_loop)]
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                // Scale the position down by the chunk size.
                let scaled_x = x as i32 + (chunk_position.x * CHUNK_SIZE as i32);
                let scaled_y = y as i32;
                let scaled_z = z as i32 + (chunk_position.z * CHUNK_SIZE as i32);

                // Sample the noise function at the scaled position.
                // The perlin noise needs a float value, so we need to cast the scaled position to a float.
                chunk_blocks[x][y][z] = is_block(IVec3::new(scaled_x, scaled_y, scaled_z), &perlin);
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

                if block_type == BlockType::Water {}

                // Check the blocks around the current block to see if we need to create faces.
                for &(x_offset, y_offset, z_offset, face) in &[
                    (0, 1, 0, BlockFace::Top),
                    (0, -1, 0, BlockFace::Bottom),
                    (1, 0, 0, BlockFace::Right),
                    (-1, 0, 0, BlockFace::Left),
                    (0, 0, 1, BlockFace::Front),
                    (0, 0, -1, BlockFace::Back),
                ] {
                    let neighbor_x = x as i32 + x_offset;
                    let neighbor_y = y as i32 + y_offset;
                    let neighbor_z = z as i32 + z_offset;

                    // Check if the neighbor block is outside the chunk.
                    if neighbor_x < 0
                        || neighbor_x >= CHUNK_SIZE as i32
                        || neighbor_y < 0
                        || neighbor_y >= CHUNK_HEIGHT as i32
                        || neighbor_z < 0
                        || neighbor_z >= CHUNK_SIZE as i32
                    {
                        // If the neighbor block is outside the chunk, we need to calculate if there is block in other chunk.
                        let neighbor_block_pos = IVec3::new(
                            x as i32 + (chunk_position.x * CHUNK_SIZE as i32) + x_offset,
                            y as i32 + y_offset,
                            z as i32 + (chunk_position.z * CHUNK_SIZE as i32) + z_offset,
                        );
                        let neighbor_block_type = is_block(neighbor_block_pos, &perlin);
                        if neighbor_block_type == BlockType::Air
                            || (block_type != BlockType::Water
                                && neighbor_block_type == BlockType::Water)
                            || (block_type != BlockType::Lava
                                && neighbor_block_type == BlockType::Lava)
                            || neighbor_block_pos.y < 0
                            || neighbor_block_pos.y > CHUNK_HEIGHT as i32
                        {
                            // Create the face.
                            create_face(
                                &mut vertices,
                                &mut indices,
                                &mut normals,
                                &mut uvs,
                                IVec2XZ::new(chunk_position.x, chunk_position.z),
                                [x as f32, y as f32, z as f32],
                                face,
                                block_type,
                                &game_texture.0.textures,
                                &game_texture.0.size,
                            );
                        }
                    } else {
                        // Get the block type of the neighbor block in the current chunk.
                        let neighbor_block_type = chunk_blocks[neighbor_x as usize]
                            [neighbor_y as usize][neighbor_z as usize];
                        // If the neighbor block is Air, we need to create a face.
                        if neighbor_block_type == BlockType::Air
                            || (block_type != BlockType::Water
                                && neighbor_block_type == BlockType::Water)
                            || (block_type != BlockType::Lava
                                && neighbor_block_type == BlockType::Lava)
                        {
                            // Create the face.
                            create_face(
                                &mut vertices,
                                &mut indices,
                                &mut normals,
                                &mut uvs,
                                IVec2XZ::new(chunk_position.x, chunk_position.z),
                                [x as f32, y as f32, z as f32],
                                face,
                                block_type,
                                &game_texture.0.textures,
                                &game_texture.0.size,
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
    info!(
        "Chunk generation @ x: {} z: {} took: {:?}",
        chunk_position.x, chunk_position.z, elapsed
    );

    chunk_mesh
}

/// Creates a face on a block.
#[allow(clippy::too_many_arguments)] // too lazy to fix
fn create_face(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    chunk_position: IVec2XZ,
    position: [f32; 3],
    direction: BlockFace,
    block: BlockType,
    textures: &[Rect],
    size: &Vec2,
) {
    // Offset the position of the face based on the chunk position.
    let position = [
        position[0] + chunk_position.x as f32 * CHUNK_SIZE as f32,
        position[1],
        position[2] + chunk_position.z as f32 * CHUNK_SIZE as f32,
    ];

    // Get the len of the verticies
    let verticies_len = vertices.len() as u32;

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
        BlockFace::Top => {
            // If this is water or lava and the face is the top, the top vert should be offset down by 0.1
            if BlockType::Water == block || BlockType::Lava == block {
                [
                    [position[0], position[1] + 0.9, position[2]],
                    [position[0], position[1] + 0.9, position[2] + 1.0],
                    [position[0] + 1.0, position[1] + 0.9, position[2] + 1.0],
                    [position[0] + 1.0, position[1] + 0.9, position[2]],
                ]
            } else {
                [
                    [position[0], position[1] + 1.0, position[2]],
                    [position[0], position[1] + 1.0, position[2] + 1.0],
                    [position[0] + 1.0, position[1] + 1.0, position[2] + 1.0],
                    [position[0] + 1.0, position[1] + 1.0, position[2]],
                ]
            }
        }
        BlockFace::Bottom => [
            [position[0], position[1], position[2] + 1.0],
            [position[0], position[1], position[2]],
            [position[0] + 1.0, position[1], position[2]],
            [position[0] + 1.0, position[1], position[2] + 1.0],
        ],

        BlockFace::Left => [
            [position[0], position[1] + 1.0, position[2] + 1.0],
            [position[0], position[1] + 1.0, position[2]],
            [position[0], position[1], position[2]],
            [position[0], position[1], position[2] + 1.0],
        ],
        BlockFace::Right => [
            [position[0] + 1.0, position[1] + 1.0, position[2]],
            [position[0] + 1.0, position[1] + 1.0, position[2] + 1.0],
            [position[0] + 1.0, position[1], position[2] + 1.0],
            [position[0] + 1.0, position[1], position[2]],
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

    let texture = match block {
        BlockType::Bedrock => textures[0],
        BlockType::Stone => textures[1],
        BlockType::Dirt => textures[2],
        BlockType::Grass => {
            if direction == BlockFace::Top {
                textures[3]
            } else if direction == BlockFace::Bottom {
                textures[2]
            } else {
                textures[4]
            }
        }
        BlockType::Log => textures[5],
        BlockType::Lava => textures[6],
        // BlockType::Lava => {
        //     if direction == BlockFace::Top {
        //         textures[6]
        //     } else {
        //         textures[1]
        //     }
        //  }
        BlockType::Water => textures[7],
        BlockType::DiamondOre => textures[11],
        BlockType::GoldOre => textures[10],
        BlockType::IronOre => textures[9],
        BlockType::CoalOre => textures[8],
        BlockType::Sand => textures[12],
        BlockType::Air => textures[0], // todo: make this not cringe
    };

    let uv = [
        [texture.min.x / size.x, texture.min.y / size.y],
        [texture.max.x / size.x, texture.min.y / size.y],
        [texture.max.x / size.x, texture.max.y / size.y],
        [texture.min.x / size.x, texture.max.y / size.y],
    ];

    // Add the UV coordinates to the vector.
    uvs.extend_from_slice(&uv);

    // Add the indices to the vector. This is clockwise order.
    indices.extend_from_slice(&[
        verticies_len,
        verticies_len + 1,
        verticies_len + 2,
        verticies_len,
        verticies_len + 2,
        verticies_len + 3,
    ]);
}

pub fn chunk_system(
    mut chunks_loaded: ResMut<ChunksLoaded>,
    mut chunk_query: Query<(Entity, &ChunkMesh)>,
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera3d>>,
    generating: Res<Generating>,
    game_atlas: Res<GameTextureAtlas>,
) {
    // Check if the world is generating.
    if !generating.0 {
        return;
    }

    let task_pool = AsyncComputeTaskPool::get();

    // Check for differences between the chunks that are loaded and the chunks that should be loaded.
    let mut chunks_to_load: HashSet<IVec2XZ> = HashSet::new();
    let mut chunks_to_unload: HashSet<IVec2XZ> = HashSet::new();

    // Get the camera position.
    let camera_position = camera_query.single().translation;

    // Calculate the player's chunk position based on their world position.
    let player_chunk_position = IVec2XZ::new(
        (camera_position.x / CHUNK_SIZE as f32).floor() as i32,
        (camera_position.z / CHUNK_SIZE as f32).floor() as i32,
    );

    // Calculate the radius of the sphere around the player.
    let radius = RENDER_DISTANCE;

    // Check for chunks to load in a circle.
    for x in -radius..=radius {
        for z in -radius..=radius {
            // Check if the chunk position is within the circle.
            if x * x + z * z <= radius * radius {
                let chunk_position = player_chunk_position + IVec2XZ::new(x, z);

                // Check if the chunk is already loaded.
                if !chunks_loaded.chunks.contains(&chunk_position) {
                    // Chunk is not loaded, add it to the list of chunks to load.
                    chunks_to_load.insert(chunk_position);
                }
            }
        }
    }

    // Check for chunks to unload in a circle.
    for loaded_chunk_position in chunks_loaded.chunks.iter() {
        let distance = *loaded_chunk_position - player_chunk_position;

        // Check if the chunk is outside the render distance.
        if distance.x * distance.x + distance.z * distance.z > radius * radius {
            chunks_to_unload.insert(*loaded_chunk_position);
        }
    }

    // Load the chunks.
    for chunk_position in chunks_to_load {
        // Spawn a new task to generate chunk mesh.
        let game_atlas = game_atlas.clone();
        let task = task_pool.spawn(async move { create_chunk_mesh(chunk_position, game_atlas) });

        // Add the task as a component to a new entity.
        commands.spawn((
            ComputeMeshTask(task),
            ChunkMesh {
                position: chunk_position,
            },
        ));

        chunks_loaded.chunks.insert(chunk_position);
    }

    // Unload the chunks.
    // info!("Unloading {} chunks", chunks_to_unload.len());
    for chunk_position in chunks_to_unload {
        // Find the entity corresponding to the chunk.
        for (entity, chunk_mesh) in chunk_query.iter_mut() {
            if chunk_mesh.position == chunk_position {
                // TODO: Make this async

                // Remove the chunk from the loaded chunks.
                chunks_loaded.chunks.retain(|&x| x != chunk_position);

                // Despawn the chunk.
                commands.entity(entity).despawn(); // TODO: Fix the warning if the chunk has been despawned already by another thread.

                break;
            }
        }
    }
}

pub fn handle_mesh_tasks(
    mut commands: Commands,
    mut mesh_tasks: Query<(Entity, &mut ComputeMeshTask)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_atlas: Res<GameTextureAtlas>,
    mut chunks_loaded: ResMut<ChunksLoaded>,
) {
    let texture = game_atlas.0.texture.clone_weak();

    for (entity, mut task) in &mut mesh_tasks {
        if let Some(chunk_mesh) = future::block_on(future::poll_once(&mut task.0)) {
            let chunk_mesh_handle: Handle<Mesh> = meshes.add(chunk_mesh);

            // Get the vertices and indices from the mesh. This is needed to create the collider.
            let (vertices, indices) = get_verts_indices(meshes.get(&chunk_mesh_handle).unwrap());

            // Check if there are vertices in the mesh.
            if vertices.is_empty() {
                // Despawn the entity.
                commands.entity(entity).despawn_recursive();

                break;
            }

            // Check if every vertice is contained in at least one chunk from the chunks that are loaded.
            // The vertices should be scaled by the chunk size.

            let chunk_position = IVec2XZ::new(
                (vertices[0].x / CHUNK_SIZE as f32).floor() as i32,
                (vertices[0].z / CHUNK_SIZE as f32).floor() as i32,
            );

            // Check if this chunk position is even loaded
            if !chunks_loaded.chunks.contains(&chunk_position) {
                // warn!("Prevented crash!");

                // Despawn the entity.
                commands.entity(entity).despawn_recursive();

                // Remove the chunk from the loaded chunks.
                chunks_loaded.chunks.retain(|&x| x != chunk_position);

                break;
            } else {
                commands
                    .entity(entity)
                    .insert(PbrBundle {
                        mesh: chunk_mesh_handle,
                        material: materials.add(StandardMaterial {
                            base_color_texture: Some(texture.clone()),
                            metallic: 1.,
                            reflectance: 1.,
                            ..default()
                        }),
                        ..Default::default()
                    })
                    .insert(Collider::trimesh(vertices, indices));

                // Task is complete, so remove task component from entity
                commands.entity(entity).remove::<ComputeMeshTask>();
            }
        }
    }
}

fn surface_generation(pos: IVec3, perlin: &Perlin) -> BlockType {
    // Sample the noise function at the scaled position.
    // The perlin noise needs a float value, so we need to cast the scaled position to a float.

    // 2d perlin noise
    // let noise_value = perlin.get([pos.x as f64 * SURFACE_SCALE, pos.z as f64 * SURFACE_SCALE]);
    // to make the terrain even more interesting, we can add more octaves of noise

    let noise_values = vec![
        perlin.get([
            pos.x as f64 * 2. * SURFACE_SCALE,
            pos.z as f64 * 2. * SURFACE_SCALE,
        ]),
        perlin.get([
            pos.x as f64 * 4. * SURFACE_SCALE,
            pos.z as f64 * 4. * SURFACE_SCALE,
        ]),
        perlin.get([
            pos.x as f64 * 6. * SURFACE_SCALE,
            pos.z as f64 * 6. * SURFACE_SCALE,
        ]),
    ];
    // add all the noise values together
    let noise_value = noise_values.iter().fold(0., |acc, &x| acc + x);
    // let noise_value32 = noise_value as f32;

    // Change values (-1, 1) -> (TERRAIN_HEIGHT, MAX_HEIGHT)
    let cieling_margin = 40; // 40 blocks from height limit
    let max_height = CHUNK_HEIGHT - cieling_margin;
    let height = remap(
        noise_value as f32,
        -1., //-1.
        12., //1. (12.)
        BLEND_HEIGHT as f32,
        max_height as f32,
    );

    // calculate block type given block position and height
    match pos.y {
        y if y == 0 => BlockType::Bedrock,
        y if y + 3 < height as i32 => BlockType::Stone,
        y if y < height as i32 && !(y > 64 && y < 72) => BlockType::Dirt,
        y if y == height as i32 && !(y > 64 && y < 72) => BlockType::Grass,
        y if y <= height as i32 && (y > 64 && y < 72) => BlockType::Sand,
        y if y > 64 && y < 70 => BlockType::Water,
        _ => BlockType::Air,
    }
}

fn cave_generation(pos: IVec3, perlin: &Perlin) -> BlockType {
    // Sample the noise function at the scaled position.
    // The perlin noise needs a float value, so we need to cast the scaled position to a float.

    // 3d perlin noise
    let cave_noise_value = perlin.get([
        pos.x as f64 * CAVE_SCALE,
        pos.y as f64 * CAVE_SCALE,
        pos.z as f64 * CAVE_SCALE,
    ]);

    // TODO: LAVA ON AIR BLOCKS BELOW Y 11
    if cave_noise_value < CAVE_THRESHOLD {
        cave_block(pos)
    } else {
        BlockType::Air
    }
}

fn cave_block(pos: IVec3) -> BlockType {
    let ore_perlin = Perlin::new(69420);
    let noise_ore_generation = ore_perlin.get([
        pos.x as f64 * ORE_SCALE,
        pos.y as f64 * ORE_SCALE,
        pos.z as f64 * ORE_SCALE,
    ]);

    // Check if the noise value is above the threshhold
    if DIAMOND_THRESHOLD.contains(&noise_ore_generation) && pos.y <= 16 {
        BlockType::DiamondOre
    } else if GOLD_THRESHOLD.contains(&noise_ore_generation) && pos.y <= 24 && pos.y >= 8 {
        BlockType::GoldOre
    } else if IRON_THRESHOLD.contains(&noise_ore_generation) && pos.y <= 48 && pos.y >= 10 {
        BlockType::IronOre
    } else if COAL_THRESHOLD.contains(&noise_ore_generation) && pos.y <= 64 && pos.y >= 24 {
        BlockType::CoalOre
    } else {
        BlockType::Stone
    }
}

fn is_block(pos: IVec3, perlin: &Perlin) -> BlockType {
    // is blocks

    // limit the world size because it will start breaking at extreme distances
    let border = 5000000;
    if pos.x >= border || pos.x < -border || pos.z >= border || pos.z < -border {
        return BlockType::Air;
    }

    // Limit the world sky
    if pos.y >= 255 {
        return BlockType::Air;
    }

    // Set bottom of the ocean
    if pos.y == 64 {
        return BlockType::Stone;
    }

    // Set bedrock
    if pos.y == 0 {
        return BlockType::Bedrock;
    }

    // Generate the 2d surface block. If it's a block, check if a cave should be generated.
    let surface_block = surface_generation(pos, perlin);
    if surface_block != BlockType::Air {
        let cave_block = cave_generation(pos, perlin);
        if cave_block == BlockType::Air {
            cave_block
        } else {
            surface_block
        }
    } else {
        surface_block
    }
}

/// The function that is used to interpolate between the noise values.
///
/// This function is used to make caves and land coexist. It's a smooth linear line from 0 to 256.
/// TODO: Implement this into is_block in a way that makes sense.
// fn noise_interpolation(y: i32) -> i32 {
//     // Linear interpolation
//     (y as f32 * 256.0 / CHUNK_HEIGHT as f32) as i32
// }

/// Remaps a value from one range to another.
fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    (value - from_min) / (from_max - from_min) * (to_max - to_min) + to_min
}

// Got this from bevy discord
// https://discord.com/channels/691052431525675048/1015147097458212864/1015147294804430848
pub fn get_verts_indices(mesh: &Mesh) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    let vertices = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        None => panic!("Mesh does not contain vertex positions"),
        Some(vertex_values) => match &vertex_values {
            VertexAttributeValues::Float32x3(positions) => positions
                .iter()
                .map(|[x, y, z]| Vec3::new(*x, *y, *z))
                .collect(),
            _ => panic!("Unexpected types in {:?}", Mesh::ATTRIBUTE_POSITION),
        },
    };

    let indices = match mesh.indices().unwrap() {
        Indices::U16(_) => {
            panic!("expected u32 indices");
        }
        Indices::U32(indices) => indices
            .chunks(3)
            .map(|chunk| [chunk[0], chunk[1], chunk[2]])
            .collect(),
    };
    (vertices, indices)
}
