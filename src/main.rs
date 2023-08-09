use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::diagnostic::SystemInformationDiagnosticsPlugin;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy_atmosphere::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_debug_lines::*;
use bevy_rapier3d::prelude::*;
use color_eyre::eyre::Result;

mod game;
use game::camera::cursor_grab_system;
use game::camera::move_player;
use game::camera::player_look;
use game::camera::spawn_player;
use game::chunk::chunk_system;
use game::chunk::handle_mesh_tasks;
use game::common::*;
use game::debug::chunk_border;
use game::debug::debug_keyboard;
use game::hud::setup_hud;
use game::hud::update_text;

fn main() -> Result<()> {
    color_eyre::install()?;

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
        // == Plugins ==
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(SystemInformationDiagnosticsPlugin)
        .add_plugins(DebugLinesPlugin::with_depth_test(true))
        .add_plugins(AtmospherePlugin)
        // Rapier
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugins(RapierDebugRenderPlugin::default())
        // == Resources ==
        .insert_resource(ChunksLoaded { chunks: vec![] })
        .insert_resource(Generating(true))
        .insert_resource(ChunkBorderToggled(true))
        .init_resource::<InputState>()
        // == Systems ==
        .add_systems(Startup, (setup, setup_hud, spawn_player))
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
    let texture_handle = asset_server.load("textures/spritesheet.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 2, 3, None, None);
    commands.insert_resource(GameTextureAtlas(texture_atlas));

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
            .looking_at(Vec3::new(-0.15, -0.05, 0.25), Vec3::new(0.3, 1.0, 0.0)),
        cascade_shadow_config,
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
