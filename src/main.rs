use std::collections::HashSet;
use std::thread::spawn;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::diagnostic::SystemInformationDiagnosticsPlugin;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy_atmosphere::prelude::*;
use bevy_flycam::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_debug_lines::*;
use bevy_rapier3d::prelude::*;
use color_eyre::eyre::Result;

mod game;
use game::chunk::chunk_system;
use game::chunk::handle_mesh_tasks;
use game::common::*;
use game::debug::chunk_border;
use game::debug::debug_keyboard;
use game::hud::setup_hud;
use game::hud::update_text;
use game::camera::*;

fn main() -> Result<()> {
    color_eyre::install()?;

    let window = WindowPlugin {
        primary_window: Some(Window {
            title: "Bevy - Voxel game".into(),
            resolution: (1280., 720.).into(),
            resizable: true,
            mode: bevy::window::WindowMode::Windowed,
            ..default()
        }),
        ..default()
    };

    App::new()
        .insert_resource(Msaa::Sample2)
        .insert_resource(PlayerPos{
            pos: Vec3::new(0.0, 0.0, 0.0),
            rot: Quat::IDENTITY,
        })
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(window),
        )
        // == Plugins ==
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(SystemInformationDiagnosticsPlugin)
        .add_plugins(DebugLinesPlugin::with_depth_test(true))
        .add_plugins(AtmospherePlugin)
        .add_plugins(NoCameraPlayerPlugin)
        // .add_plugins(RapierDebugRenderPlugin
        //     {
        //         enabled: true,
        //         mode: bevy_rapier3d::render::DebugRenderMode::COLLIDER_AABBS,
        //         ..Default::default()
        //     }
        // )
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 24.0,         // default: 12.0
        })
        // Rapier
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugins(RapierDebugRenderPlugin::default())
        // == Resources ==
        .insert_resource(ChunksLoaded {
            chunks: HashSet::new(),
        })
        .insert_resource(Generating(true))
        .insert_resource(ChunkBorderToggled(true))
        .init_resource::<InputState>()
        // == Systems ==
        .add_systems(Startup, (setup, setup_hud,spawn_player))
        .add_systems(
            Update,
            (
                chunk_border,
                debug_keyboard,
                update_text,
                chunk_system,
                handle_mesh_tasks,
                cursor_grab_system,
                move_player,
                player_look,
                update_camera
            ),
        )
        .run();

    Ok(())
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut grav_scale: Query<&mut GravityScale>,
) {
    // Setup texture atlas
    let texture_handle = asset_server.load("textures/blocks.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 7, 7, None, None); //c2 r3
    commands.insert_resource(GameTextureAtlas(texture_atlas));

    // Sun
    let sun_light: f32 = 0.8;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(0.98 * sun_light, 0.95 * sun_light, 0.82 * sun_light), //r0.98 g0.95 b0.82
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .looking_at(Vec3::new(-0.15, -0.05, 0.25), Vec3::new(0.3, 1.0, 0.0)),

        ..default()
    });

    // Setup gravity
    for mut grav in grav_scale.iter_mut() {
        grav.0 = 1.0;
    }

    
}

// this is in tests.rs
#[cfg(test)]
mod tests;
