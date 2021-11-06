#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Size {
        Size { width, height }
    }
}

impl From<[f32; 2]> for Size {
    fn from(lhs: [f32; 2]) -> Size {
        Size::new(lhs[0], lhs[1])
    }
}

impl From<(f32, f32)> for Size {
    fn from(lhs: (f32, f32)) -> Size {
        Size::new(lhs.0, lhs.1)
    }
}
