use bevy::prelude::*;
use bevy_prototype_debug_lines::DebugLines;

use crate::common::*;

pub fn debug_keyboard(
    keyboard_input: Res<Input<KeyCode>>,
    mut generating: ResMut<Generating>,
    mut commands: Commands,
    chunk_query: Query<Entity, With<ChunkMesh>>,
    mut chunks_loaded: ResMut<ChunksLoaded>,
    mut chunk_border_toggled: ResMut<ChunkBorderToggled>,
) {
    if keyboard_input.just_pressed(KeyCode::P) {
        // Toggle the generating resource.
        generating.0 = !generating.0;
    }
    if keyboard_input.just_pressed(KeyCode::R) {
        // Delete all chunks.
        for entity in chunk_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        chunks_loaded.chunks = vec![];
    }
    if keyboard_input.just_pressed(KeyCode::G) {
        // Toggle the chunk border.
        chunk_border_toggled.0 = !chunk_border_toggled.0;
    }
}

pub fn chunk_border(
    // chunk_position: IVec3,
    // chunk_borders: Query<&ChunkBorder>,
    mut lines: ResMut<DebugLines>,
    camera: Query<&Transform, With<Camera>>,
    chunk_border_toggled: Res<ChunkBorderToggled>,
) {
    // Check if the chunk border should be drawn.
    if !chunk_border_toggled.0 {
        return;
    }

    // Draw a "box" around the selected chunk.
    // Determine the current from the camera position
    let camera_position = camera.single().translation;
    let current_chunk: IVec2XZ = IVec2XZ::new(
        (camera_position.x / CHUNK_SIZE as f32).floor() as i32,
        (camera_position.z / CHUNK_SIZE as f32).floor() as i32,
    );

    // Draw the lines around the current chunk.
    let x1 = current_chunk.x * CHUNK_SIZE as i32;
    let y1 = 0;
    let z1 = current_chunk.z * CHUNK_SIZE as i32;

    let x2 = x1 + CHUNK_SIZE as i32;
    let y2 = CHUNK_HEIGHT as i32;
    let z2 = z1 + CHUNK_SIZE as i32;

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
