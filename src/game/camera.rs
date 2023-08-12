// THIS FILE IS CURRENTLY UNUSED!
// A big part of this is thanks to the bevy_flycam crate
use bevy::{input::mouse::MouseMotion, pbr::NotShadowCaster, prelude::*, window::PrimaryWindow};
use bevy_atmosphere::prelude::AtmosphereCamera;
use bevy_flycam::prelude::*;
use bevy_rapier3d::{prelude::*, rapier::prelude::{RigidBodyHandle, RigidBodySet}};

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
            NotShadowCaster,
        ))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 200.0, 0.0)));

    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, 200.0, 0.0)),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED_Z
            | LockedAxes::ROTATION_LOCKED_X
            | LockedAxes::ROTATION_LOCKED_Y,
        Collider::capsule_y(0.5, 0.5),
        // Collider::cuboid(0.5, 1.0, 0.5),
        Sleeping::disabled(),
        Ccd::enabled(),

    )).insert(KinematicCharacterController {
        offset: CharacterLength::Absolute(0.1),
        up: Vec3::Y,
        autostep: None,
        ..default()
    });
}

// todo: make the query more readable
pub fn move_player(
    mut controllers: Query<(&mut KinematicCharacterController, &mut Transform)>,
    // mut camera: Query<(&Camera3d, &mut Transform)>,
    ground_touching: Query<&KinematicCharacterControllerOutput>,
    keys: Res<Input<KeyCode>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut player_state: ResMut<PlayerPos>,
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
    let mut sprinting = false;

    for key in keys.get_pressed() {
        match key {
            KeyCode::W => new_translation -= forward,
            KeyCode::S => new_translation += forward,
            KeyCode::A => new_translation -= right,
            KeyCode::D => new_translation += right,
            KeyCode::ShiftLeft => sprinting = true,
            _ => (),
        }
    }

    // Normalize so that diagonal movement isn't faster
    new_translation = new_translation.normalize_or_zero();

    // Sprinting
    if sprinting {
        new_translation *= 2.0;
    }

    // Scale by time
    new_translation *= time.delta_seconds() * SPEED;

    // Gravity is scaled by GRAVITY
    // new_translation.y -= time.delta_seconds() * GRAVITY;

    // Jumping
    if keys.just_pressed(KeyCode::Space) {
        // for output in ground_touching.iter() {
            // if output.grounded {
                new_translation.y += JUMP_FORCE;
            // }
        // }
    }

    for mut controller in controllers.iter_mut() {
        controller.0.translation = Some(new_translation);
        controller.1.rotation = player_state.rot;
        player_state.pos = controller.1.translation;
    }
}

pub fn update_camera(
    mut camera: Query<(&mut Transform, &AtmosphereCamera)>,
    player_state: Res<PlayerPos>,
) {
    for (mut transform, _) in camera.iter_mut() {
        transform.translation = Vec3::new(
            player_state.pos.x,
            player_state.pos.y + 0.5,
            player_state.pos.z,
        );
    }
}

pub fn player_look(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    // mut query: Query<&mut Transform, &KinematicCharacterController>,
    mut camera: Query<&mut Transform, With<AtmosphereCamera>>,
    mut player_state: ResMut<PlayerPos>,
) {
    if let Ok(window) = primary_window.get_single() {
        for mut transform in camera.iter_mut() {
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

                player_state.rot = Quat::from_axis_angle(Vec3::Y, yaw);
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
