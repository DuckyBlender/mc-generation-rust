use bevy::{
    diagnostic::{
        DiagnosticsStore, FrameTimeDiagnosticsPlugin, SystemInformationDiagnosticsPlugin,
    },
    prelude::*,
};

use crate::common::*;

// For FPS counter
#[derive(Component)]
pub struct TextChanges;

/// Updates the UI text.
///
/// Information about the FPS, coordinates and direction is displayed.
pub fn update_text(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<TextChanges>>,
    camera_query: Query<&Transform, With<Camera>>,
    chunk_query: Query<&ChunkMesh>,
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

    let chunks_loaded = chunk_query.iter().count();

    // Update the coordinates and direction.
    let camera_transform = camera_query.single();
    let camera_transform_chunks: IVec2XZ = IVec2XZ::new(
        (camera_transform.translation.x / CHUNK_SIZE as f32).floor() as i32,
        (camera_transform.translation.z / CHUNK_SIZE as f32).floor() as i32,
    );

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
        "FPS: {:.2}, CPU: {:.2}%, RAM: {:.2}%\nChunks loaded: {}\n\nPosition: ({:.2}, {:.2}, {:.2}) Chunk: ({}, {})\nDirection: {}",
        fps,
        cpu,
        ram,
        chunks_loaded,
        camera_position.x,
        camera_position.y,
        camera_position.z,
        camera_transform_chunks.x,
        camera_transform_chunks.z,
        direction
    );
}

pub fn setup_hud(mut commands: Commands) {
    // Manual implementation of the crosshair.
    // root node
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            UI,
        ))
        .with_children(|parent| {
            parent.spawn((NodeBundle {
                style: Style {
                    width: Val::Px(2.0),
                    height: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    align_content: AlignContent::Center,
                    align_self: AlignSelf::Center,
                    position_type: PositionType::Absolute,

                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(1.0, 0.0, 0.0, 1.0)),
                ..default()
            },));
            parent.spawn((NodeBundle {
                style: Style {
                    width: Val::Px(10.0),
                    height: Val::Px(2.0),
                    align_items: AlignItems::Center,
                    align_content: AlignContent::Center,
                    align_self: AlignSelf::Center,
                    position_type: PositionType::Absolute,
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(1.0, 0.0, 0.0, 1.0)),
                ..default()
            },));
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
            left: Val::Px(10.0),
            ..default()
        }),
        TextChanges,
    ));

    // Text to display controls
    commands.spawn((TextBundle::from_section(
        "P - Pause Chunk generation\nR - Reset Chunks\nG - Toggle Chunks Borders".to_string(),
        TextStyle {
            font_size: 20.0,
            ..default()
        },
    )
    .with_style(Style {
        position_type: PositionType::Absolute,
        bottom: Val::Px(20.0),
        right: Val::Px(10.0),
        ..default()
    }),));
}
