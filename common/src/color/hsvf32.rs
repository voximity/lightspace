/// 32-bit floating point HSV.
#[derive(Debug, Clone, Copy)]
pub struct HsvF32 {
    /// [0, 360] (any number)
    pub hue: f32,
    /// [0, 1]
    pub saturation: f32,
    /// [0, 1]
    pub value: f32,
}

impl HsvF32 {
    pub fn new(hue: f32, saturation: f32, value: f32) -> Self {
        Self {
            hue,
            saturation,
            value,
        }
    }

    pub fn zero() -> Self {
        Self {
            hue: 0f32,
            saturation: 0f32,
            value: 0f32,
        }
    }
}
