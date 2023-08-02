use bevy::prelude::*;
use bevy_prototype_debug_lines::DebugLines;

use crate::common::*;

pub fn debug_keyboard(keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::F) {
        info!("F");
    }
}

pub fn chunk_border(
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
