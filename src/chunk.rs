use std::time::Instant;

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};
use noise::{NoiseFn, Perlin};

use crate::{common::*, BlockType, ChunksLoaded};

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
                let scaled_y = y as i32;
                let scaled_z = z as i32 + (chunk_position.z * CHUNK_SIZE as i32);
                // info!("Scaled position: {}, {}, {}", scaled_x, scaled_y, scaled_z);

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
                        let neighbor_block_type = is_block(
                            IVec3::new(
                                x as i32 + (chunk_position.x * CHUNK_SIZE as i32) + x_offset,
                                y as i32 + y_offset,
                                z as i32 + (chunk_position.z * CHUNK_SIZE as i32) + z_offset,
                            ),
                            &perlin,
                        );
                        if neighbor_block_type == BlockType::Air {
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
                        if neighbor_block_type == BlockType::Air {
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
    info!("Chunk generation took: {:?}", elapsed);

    chunk_mesh
}

/// Creates a face on a block.
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
            } else {
                textures[4]
            }
        }
        BlockType::Log => textures[5],
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
        first_vertex,
        first_vertex + 1,
        first_vertex + 2,
        first_vertex,
        first_vertex + 2,
        first_vertex + 3,
    ]);
}

pub fn chunk_system(
    mut commands: Commands,
    mut chunks_loaded: ResMut<ChunksLoaded>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<&Transform, With<Camera3d>>,
    generating: Res<Generating>,
    game_atlas: Res<GameTextureAtlas>,
) {
    // Check if the world is generating.
    if !generating.0 {
        return;
    }

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
    let texture = game_atlas.0.texture.clone_weak();

    for chunk_position in chunks_to_load {
        // Create thread to generate chunk mesh.

        let chunk_mesh_handle: Handle<Mesh> =
            meshes.add(create_chunk_mesh(chunk_position, game_atlas.clone()));

        commands.spawn((
            PbrBundle {
                mesh: chunk_mesh_handle,
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(texture.clone()),

                    ..Default::default()
                }),
                ..Default::default()
            },
            ChunkMesh,
        ));

        chunks_loaded.chunks.push(chunk_position);
    }
}

fn is_block_2d(pos: IVec3, perlin: &Perlin) -> BlockType {
    // Sample the noise function at the scaled position.
    // The perlin noise needs a float value, so we need to cast the scaled position to a float.
    // TODO: Implement more 2d noise functions for more varied terrain.

    // 2d perlin noise
    let noise_value = perlin.get([pos.x as f64 * WORLD_SCALE, pos.z as f64 * WORLD_SCALE]);

    // Change values (-1, 1) -> (TERRAIN_HEIGHT, MAX_HEIGHT)
    let cieling_margin = 80;
    let max_height = CHUNK_HEIGHT - cieling_margin;
    let height = remap(
        noise_value as f32,
        -1.,
        1.,
        TERRAIN_HEIGHT as f32,
        max_height as f32,
    );

    // calculate block type given block position and height
    match pos.y {
        y if y == 0 => BlockType::Bedrock,
        y if y < height as i32 => BlockType::Dirt,
        y if y == height as i32 => BlockType::Grass,
        _ => BlockType::Air,
    }
}

fn is_block_3d(pos: IVec3, perlin: &Perlin) -> BlockType {
    // Sample the noise function at the scaled position.
    // The perlin noise needs a float value, so we need to cast the scaled position to a float.

    // 3d perlin noise
    let noise_value = perlin.get([
        pos.x as f64 * WORLD_SCALE,
        pos.y as f64 * WORLD_SCALE,
        pos.z as f64 * WORLD_SCALE,
    ]);

    // If the noise value is above the threshold, create a cube at that position.
    if noise_value < NOISE_THRESHOLD {
        BlockType::Stone
    } else {
        BlockType::Air
    }
}

fn is_block(pos: IVec3, perlin: &Perlin) -> BlockType {
    // If the block is at y0, create a bedrock block.
    if pos.y == 0 {
        return BlockType::Bedrock;
    }

    let two_noise_value = is_block_2d(pos, perlin);
    let three_noise_value = is_block_3d(pos, perlin);

    if pos.y > TERRAIN_HEIGHT {
        two_noise_value
    } else {
        three_noise_value
    }
}

/// The function that is used to interpolate between the noise values.
///
/// This function is used to make caves and land coexist. It's a smooth linear line from 0 to 256.
/// TODO: Implement this into is_block in a way that makes sense.
fn noise_interpolation(y: i32) -> i32 {
    // Linear interpolation
    (y as f32 * 256.0 / CHUNK_HEIGHT as f32) as i32
}

/// Remaps a value from one range to another.
fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    (value - from_min) / (from_max - from_min) * (to_max - to_min) + to_min
}
