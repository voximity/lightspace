#![cfg_attr(not(feature = "std"), no_std)]

pub mod format;
pub mod math;

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
