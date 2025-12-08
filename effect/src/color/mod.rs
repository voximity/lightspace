pub mod hsvf32;
pub mod rgb8;
pub mod rgbf32;

pub use hsvf32::HsvF32;
pub use rgb8::Rgb8;
pub use rgbf32::RgbF32;

pub trait MapColor: Copy {
    type Component;

    fn map<F>(self, f: F) -> Self
    where
        F: Fn(Self::Component) -> Self::Component;
}

pub trait ZipColor: MapColor {
    fn zip<F>(self, other: Self, f: F) -> Self
    where
        F: Fn(Self::Component, Self::Component) -> Self::Component;
}
