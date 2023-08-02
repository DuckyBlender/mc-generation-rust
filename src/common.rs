use bevy::prelude::*;

pub const RENDER_DISTANCE: i32 = 6;
pub const SEED: u32 = 2137;
pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;
pub const WORLD_SCALE: f64 = 0.1;
pub const NOISE_THRESHOLD: f64 = 0.3;

// === COMPONENTS ===

#[derive(Component)]
pub struct ChunkMesh;

#[derive(Component)]
pub struct ChunkBorder;

#[derive(Component)]
pub struct UI;

// === RESOURCES ===

#[derive(Resource)]
pub struct ChunksLoaded {
    pub chunks: Vec<IVec2XZ>,
}

// === ENUMS ===

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum BlockType {
    Air,
    Dirt,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
