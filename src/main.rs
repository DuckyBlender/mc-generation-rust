use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::diagnostic::SystemInformationDiagnosticsPlugin;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy_atmosphere::prelude::*;
use bevy_flycam::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_debug_lines::*;
use bevy_rapier3d::prelude::*;

mod common;
use common::*;

mod hud;
use hud::*;

mod debug;
use debug::*;

mod chunk;
use chunk::*;

fn main() {
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
        .add_plugins(NoCameraPlayerPlugin)
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 30.0,          // default: 12.0
        })
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
        // == Systems ==
        .add_systems(Startup, (setup, setup_hud))
        .add_systems(
            Update,
            (
                chunk_border,
                debug_keyboard,
                update_text,
                chunk_system,
                handle_mesh_tasks,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Setup texture atlas
    let texture_handle = asset_server.load("textures/spritesheet.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 2, 3, None, None);
    commands.insert_resource(GameTextureAtlas(texture_atlas));

    // Camera in 3D space.
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 200.0, 0.0))
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
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
        FlyCam,
        AtmosphereCamera::default(),
    ));

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
            .looking_at(Vec3::new(-0.15, -0.05, 0.25), Vec3::Y),
        cascade_shadow_config,
        ..default()
    });
}

// this is in tests.rs
#[cfg(test)]
mod tests;
