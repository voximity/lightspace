#[cfg(feature = "firmware")]
use num_traits::Float;

/// Linear interpolation between two floats.
#[inline(always)]
pub fn lerp(a: f32, b: f32, c: f32) -> f32 {
    a + (b - a) * c
}

/// Convert a [0..1] f32 to a [0..255] u8.
#[inline(always)]
pub fn f32_to_u8(x: f32) -> u8 {
    (x * 255.0).round().clamp(0.0, 255.0) as u8
}
