use crate::color::{Rgb8, RgbF32};

/// Pre-multiplied alpha floating point color.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct RgbaF32 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl RgbaF32 {
    /// Create a pre-multiplied RGBA from non-premultiplied components.
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: r * a,
            g: g * a,
            b: b * a,
            a,
        }
    }

    /// Create a pre-multiplied RGBA from premultiplied components.
    pub const fn new_premultiplied(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// All-components-zero RGBA.
    pub const fn zero() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }
    }

    /// Convert a RGB to an RGBA, given an A.
    pub fn from_rgb(rgb: RgbF32, a: f32) -> Self {
        Self::new(rgb.r, rgb.g, rgb.b, a)
    }

    /// Blend this RGBA over another.
    pub fn blend_over(self, bg: Self) -> Self {
        let inv_as = 1.0 - self.a;
        Self {
            r: self.r + bg.r * inv_as,
            g: self.g + bg.g * inv_as,
            b: self.b + bg.b * inv_as,
            a: self.a + bg.a * inv_as,
        }
    }
}

impl From<RgbF32> for RgbaF32 {
    fn from(value: RgbF32) -> Self {
        Self::from_rgb(value, 1.0)
    }
}

impl From<Rgb8> for RgbaF32 {
    fn from(value: Rgb8) -> Self {
        Self::from_rgb(RgbF32::from(value), 1.0)
    }
}
