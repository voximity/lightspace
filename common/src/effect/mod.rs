use crate::color::{HsvF32, Rgb8, RgbF32};

#[cfg(feature = "firmware")]
use num_traits::{Euclid, Float};

#[derive(Debug, Default, Clone, Copy)]
pub struct StripInfo {
    pub leds: usize,
    pub rev: bool,
    // TODO: spatial data
}

impl StripInfo {
    pub const fn empty() -> Self {
        Self {
            leds: 0,
            rev: false,
        }
    }
}

pub trait EffectMode {
    fn update(&self, info: &StripInfo, buf: &mut [Rgb8], time: u64);
}

fn rem_euclid(a: f32, b: f32) -> f32 {
    #[cfg(feature = "firmware")]
    return a.rem_euclid(&b);
    #[cfg(not(feature = "firmware"))]
    a.rem_euclid(b)
}

#[derive(better_default::Default)]
pub struct ColorWheel {
    #[default(1.0)]
    pub saturation: f32,

    #[default(1.0)]
    pub value: f32,

    #[default(1.0 / 500.0)]
    pub deg_per_px: f32,

    #[default(30.0)]
    pub deg_per_sec: f32,
}

impl EffectMode for ColorWheel {
    fn update(&self, _: &StripInfo, buf: &mut [Rgb8], time: u64) {
        for (i, px) in buf.iter_mut().enumerate() {
            let hsv = HsvF32::new(
                rem_euclid(
                    (i as f32 * self.deg_per_px) * -360.0
                        + (time as f32 / 1000.0 * self.deg_per_sec),
                    360.0,
                ),
                self.saturation,
                self.value,
            );

            *px = Rgb8::from(RgbF32::from(hsv)).gamma_correct();
        }
    }
}

pub struct ColorPattern<const N: usize> {
    pub colors: [Rgb8; N],
    pub speed: f32,
}

impl<const N: usize> EffectMode for ColorPattern<N> {
    fn update(&self, _: &StripInfo, buf: &mut [Rgb8], time: u64) {
        let time_shift = (time as f32 / 1000.0 * self.speed * N as f32).floor() as usize;
        for (i, px) in buf.iter_mut().enumerate() {
            *px = self.colors[(i - time_shift).rem_euclid(N)].gamma_correct();
        }
    }
}

pub struct Bounce {
    pub color: Rgb8,
    pub speed: f32,
}

impl EffectMode for Bounce {
    fn update(&self, info: &StripInfo, buf: &mut [Rgb8], time: u64) {
        let cur = (((time as f32 / 1000.0 * core::f32::consts::PI * 2.0 * self.speed).sin())
            * (info.leds as f32 / 2.0)
            + (info.leds as f32 / 2.0))
            .floor() as usize;
        for (i, px) in buf.iter_mut().enumerate() {
            if i == cur {
                *px = self.color;
            } else {
                *px = Rgb8::zero();
            }
        }
    }
}
