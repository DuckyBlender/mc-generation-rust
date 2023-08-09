use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_prototype_debug_lines::DebugLines;

use crate::common::*;

pub fn debug_keyboard(
    keyboard_input: Res<Input<KeyCode>>,
    mut generating: ResMut<Generating>,
    mut commands: Commands,
    chunk_query: Query<Entity, With<ChunkMesh>>,
    mut chunks_loaded: ResMut<ChunksLoaded>,
    mut chunk_border_toggled: ResMut<ChunkBorderToggled>,
    mut windows: Query<&mut Window>,
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
    // Toggle VSync
    if keyboard_input.just_pressed(KeyCode::V) {
        let mut window = windows.single_mut();

        window.present_mode = if matches!(window.present_mode, PresentMode::AutoVsync) {
            PresentMode::AutoNoVsync
        } else {
            PresentMode::AutoVsync
        };
        info!("Window Mode: {:?}", window.present_mode);
    }
}

pub fn chunk_border(
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

    let duration = 0.0;
    let color = Color::rgb(1.0, 0.0, 0.0);

    let corners = [
        [x1, y1, z1],
        [x2, y1, z1],
        [x1, y2, z1],
        [x1, y1, z2],
        [x2, y2, z1],
        [x2, y1, z2],
        [x1, y2, z2],
        [x2, y2, z2],
    ];

    // Convert corners to Vec3
    let corners: Vec<Vec3> = corners
        .iter()
        .map(|corner| Vec3::new(corner[0] as f32, corner[1] as f32, corner[2] as f32))
        .collect();

    for i in 0..corners.len() {
        let j = (i + 1) % corners.len();
        lines.line_colored(corners[i], corners[j], duration, color);
    }
}
