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
