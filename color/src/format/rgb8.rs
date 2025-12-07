use core::ops::Mul;

use crate::{MapColor, format::RgbF32, math::f32_to_u8};

#[cfg(feature = "num-traits")]
use num_traits::Float;

pub const GAMMA: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5,
    5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 13, 13, 13, 14,
    14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23, 24, 24, 25, 25, 26, 27,
    27, 28, 29, 29, 30, 31, 32, 32, 33, 34, 35, 35, 36, 37, 38, 39, 39, 40, 41, 42, 43, 44, 45, 46,
    47, 48, 49, 50, 50, 51, 52, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 66, 67, 68, 69, 70, 72,
    73, 74, 75, 77, 78, 79, 81, 82, 83, 85, 86, 87, 89, 90, 92, 93, 95, 96, 98, 99, 101, 102, 104,
    105, 107, 109, 110, 112, 114, 115, 117, 119, 120, 122, 124, 126, 127, 129, 131, 133, 135, 137,
    138, 140, 142, 144, 146, 148, 150, 152, 154, 156, 158, 160, 162, 164, 167, 169, 171, 173, 175,
    177, 180, 182, 184, 186, 189, 191, 193, 196, 198, 200, 203, 205, 208, 210, 213, 215, 218, 220,
    223, 225, 228, 231, 233, 236, 239, 241, 244, 247, 249, 252, 255,
];

/// 8-bit sRGB.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Rgb8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb8 {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const fn gray(x: u8) -> Self {
        Self { r: x, g: x, b: x }
    }

    pub fn gamma_correct(self) -> Self {
        self.map(|c| GAMMA[c as usize])
    }

    pub fn brightness(self, x: f32) -> Self {
        self * x
    }
}

impl MapColor for Rgb8 {
    type Component = u8;

    fn map<F>(self, f: F) -> Self
    where
        F: Fn(u8) -> u8,
    {
        Self {
            r: f(self.r),
            g: f(self.g),
            b: f(self.b),
        }
    }
}

impl From<RgbF32> for Rgb8 {
    fn from(value: RgbF32) -> Self {
        Self {
            r: f32_to_u8(value.r),
            g: f32_to_u8(value.g),
            b: f32_to_u8(value.b),
        }
    }
}

impl Mul<f32> for Rgb8 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        self.map(|c| (c as f32 * rhs).clamp(0f32, 255f32).round() as u8)
    }
}
