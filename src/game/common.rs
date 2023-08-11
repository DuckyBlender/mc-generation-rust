use bevy::{ecs::event::ManualEventReader, input::mouse::MouseMotion, prelude::*, tasks::Task};
use std::collections::HashSet;

pub const RENDER_DISTANCE: i32 = 12;
pub const SEED: u32 = 2137;
pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;
pub const SURFACE_SCALE: f64 = 0.004; //0.008
pub const CAVE_SCALE: f64 = 0.06; //0.06
pub const NOISE_THRESHOLD: f64 = 0.32; //0.3
pub const TERRAIN_HEIGHT: i32 = 64; //160
pub const FOV: f32 = 80.0;

// pub const SPEED: f32 = 10.0;
// pub const GRAVITY: f32 = 9.81;
// pub const JUMP_FORCE: f32 = 2.5;

// === COMPONENTS ===

#[derive(Component)]
pub struct ChunkMesh {
    pub position: IVec2XZ,
}

#[derive(Component)]
pub struct ComputeMeshTask(pub Task<Mesh>);

#[derive(Component)]
pub struct ChunkBorder;

#[derive(Component)]
pub struct UI;

// === RESOURCES ===

#[derive(Resource)]
pub struct ChunksLoaded {
    pub chunks: HashSet<IVec2XZ>,
}

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Resource, Default)]
pub struct InputState {
    pub reader_motion: ManualEventReader<MouseMotion>,
}

#[derive(Resource)]
pub struct ChunkBorderToggled(pub bool);

#[derive(Resource)]
pub struct Generating(pub bool);

#[derive(Resource, Clone)]
pub struct GameTextureAtlas(pub TextureAtlas);

// === ENUMS ===

#[derive(PartialEq, Copy, Clone, Default)]
pub enum BlockType {
    Bedrock,
    Stone,
    Dirt,
    Grass,
    Log,
    Lava,
    Water,
    Diamond_ore,
    Gold_ore,
    Iron_ore,
    Coal_ore,
    Sand,
    #[default]
    Air,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlockFace {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

// === IVEC2XZ ===

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct IVec2XZ {
    pub x: i32,
    pub z: i32,
}

impl IVec2XZ {
    pub fn new(x: i32, z: i32) -> Self {
        IVec2XZ { x, z }
    }
}

impl std::ops::Add for IVec2XZ {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.z + rhs.z)
    }
}

impl std::ops::Sub for IVec2XZ {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.z - rhs.z)
    }
}
