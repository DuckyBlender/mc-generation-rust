// A big part of this is thanks to the bevy_flycam crate

use bevy::{input::mouse::MouseMotion, prelude::*, window::PrimaryWindow};
use bevy_atmosphere::prelude::AtmosphereCamera;
use bevy_rapier3d::prelude::*;

use bevy::window::CursorGrabMode;

use crate::game::common::*;

pub fn spawn_player(mut commands: Commands) {
    // Spawn with rectangle collision
    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_translation(Vec3::new(0.0, 200.0, 0.0))
                    .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: FOV.to_radians(),
                    ..default()
                }),
                ..default()
            },
            FogSettings {
                color: Color::rgba(0.05, 0.05, 0.05, 1.0),
                falloff: FogFalloff::Linear {
                    start: RENDER_DISTANCE as f32 * CHUNK_SIZE as f32 * 0.8,
                    end: RENDER_DISTANCE as f32 * CHUNK_SIZE as f32 * 0.95,
                },
                ..default()
            },
            AtmosphereCamera::default(),
            RigidBody::KinematicPositionBased,
        ))
        .insert(KinematicCharacterController::default())
        .insert(Collider::cuboid(0.5, 1.0, 0.5))
        // .insert(AdditionalMassProperties::Mass(10.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 200.0, 0.0)))
        .insert(Sleeping::disabled()) // Disable sleeping so that the player doesn't fall through the ground
        .insert(Ccd::enabled()); // Continuous collision detection;
}

// todo: make the query more readable
pub fn move_player(
    mut controllers: Query<(&mut KinematicCharacterController, &Transform)>,
    keys: Res<Input<KeyCode>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    if primary_window.get_single().is_err() {
        return;
    }

    let mut new_translation = Vec3::new(0.0, 0.0, 0.0);

    // Local z is the direction the player is facing
    let local_z = controllers
        .iter_mut()
        .next()
        .unwrap()
        .1
        .rotation
        .mul_vec3(Vec3::Z);
    let forward = Vec3::new(local_z.x, 0.0, local_z.z).normalize();
    let right = Vec3::new(local_z.z, 0.0, -local_z.x).normalize();

    for key in keys.get_pressed() {
        match key {
            KeyCode::W => new_translation += forward,
            KeyCode::S => new_translation -= forward,
            KeyCode::A => new_translation += right,
            KeyCode::D => new_translation -= right,
            _ => (),
        }
    }

    // Normalize so that diagonal movement isn't faster
    if new_translation.length() > 0.0 {
        new_translation = new_translation.normalize();
    }

    // Transform from global to local coordinates
    new_translation = Quat::from_rotation_y(std::f32::consts::PI) * new_translation * 0.1; // 0.1 is the speed

    for mut controller in controllers.iter_mut() {
        controller.0.translation = Some(new_translation);
    }
}

pub fn player_look(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<&mut Transform, &KinematicCharacterController>,
) {
    if let Ok(window) = primary_window.get_single() {
        for mut transform in query.iter_mut() {
            for mouse_motion in state.reader_motion.iter(&motion) {
                let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                        let window_scale = window.height().min(window.width());
                        pitch -= (0.0005 * mouse_motion.delta.y * window_scale).to_radians();
                        yaw -= (0.0005 * mouse_motion.delta.x * window_scale).to_radians();
                    }
                }

                pitch = pitch.clamp(-1.54, 1.54);

                // Order is important to prevent unintended roll
                transform.rotation =
                    Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
            }
        }
    } else {
        warn!("Primary window not found for `player_look`!");
    }
}

pub fn cursor_grab_system(
    mut window: Query<&mut Window>,
    button: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let mut window = window.single_mut();
    let window = &mut *window;

    // Toggle cursor grab mode and visibility.
    if button.just_pressed(MouseButton::Left) {
        window.cursor.grab_mode = CursorGrabMode::Confined;
        window.cursor.visible = false;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }
}
