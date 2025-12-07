use core::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

use crate::{MapColor, ZipColor, format::HsvF32, math::lerp};

#[cfg(feature = "num-traits")]
use num_traits::Float;

/// 32-bit floating point sRGB.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RgbF32 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl RgbF32 {
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    pub const fn gray(x: f32) -> Self {
        Self { r: x, g: x, b: x }
    }

    pub fn lerp(self, other: Self, delta: f32) -> Self {
        Self {
            r: lerp(self.r, other.r, delta),
            g: lerp(self.g, other.g, delta),
            b: lerp(self.b, other.b, delta),
        }
    }
}

impl MapColor for RgbF32 {
    type Component = f32;

    fn map<F>(self, f: F) -> Self
    where
        F: Fn(f32) -> f32,
    {
        Self {
            r: f(self.r),
            g: f(self.g),
            b: f(self.b),
        }
    }
}

impl ZipColor for RgbF32 {
    fn zip<F>(self, other: Self, f: F) -> Self
    where
        F: Fn(Self::Component, Self::Component) -> Self::Component,
    {
        Self {
            r: f(self.r, other.r),
            g: f(self.g, other.g),
            b: f(self.b, other.b),
        }
    }
}

impl Add for RgbF32 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.zip(rhs, |a, b| 1f32.min(a + b))
    }
}

impl AddAssign for RgbF32 {
    fn add_assign(&mut self, rhs: Self) {
        self.r = 1f32.min(self.r + rhs.r);
        self.g = 1f32.min(self.g + rhs.g);
        self.b = 1f32.min(self.b + rhs.b);
    }
}

impl Sub for RgbF32 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.zip(rhs, |a, b| 0f32.max(a - b))
    }
}

impl SubAssign for RgbF32 {
    fn sub_assign(&mut self, rhs: Self) {
        self.r = 0f32.max(self.r + rhs.r);
        self.g = 0f32.max(self.g + rhs.g);
        self.b = 0f32.max(self.b + rhs.b);
    }
}

impl Mul<f32> for RgbF32 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        self.map(|a| (a * rhs).clamp(0f32, 1f32))
    }
}

impl Mul for RgbF32 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.zip(rhs, |a, b| (a * b).clamp(0f32, 1f32))
    }
}

impl MulAssign<f32> for RgbF32 {
    fn mul_assign(&mut self, rhs: f32) {
        self.r = (self.r * rhs).clamp(0f32, 1f32);
        self.g = (self.g * rhs).clamp(0f32, 1f32);
        self.b = (self.b * rhs).clamp(0f32, 1f32);
    }
}

impl MulAssign for RgbF32 {
    fn mul_assign(&mut self, rhs: Self) {
        self.r = (self.r * rhs.r).clamp(0f32, 1f32);
        self.g = (self.g * rhs.g).clamp(0f32, 1f32);
        self.b = (self.b * rhs.b).clamp(0f32, 1f32);
    }
}

impl From<HsvF32> for RgbF32 {
    fn from(hsv: HsvF32) -> Self {
        let h = hsv.hue;
        let s = hsv.saturation.clamp(0.0, 1.0);
        let v = hsv.value.clamp(0.0, 1.0);

        if s == 0.0 {
            return RgbF32::gray(v);
        }

        let c = v * s;
        let hh = h / 60.0;
        let x = c * (1.0 - ((hh % 2.0) - 1.0).abs());

        let (r1, g1, b1) = match hh.floor() as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        let m = v - c;

        RgbF32 {
            r: r1 + m,
            g: g1 + m,
            b: b1 + m,
        }
    }
}
