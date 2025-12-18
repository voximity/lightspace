use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};

/// A TCP message from the server to a node.
#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    /// Set a strip's mode, given a [`StripMode`].
    SetStripMode(u8, StripMode),
    /// Shift the current effect mode by a delta.
    ShiftEffectMode(i8),
}

/// A TCP message from a node to the server.
pub enum NodeMessage {}

/// A UDP message from the server to a node.
#[derive(TryFromPrimitive)]
#[repr(u8)]
pub enum UdpMessage {
    /// Set the entire buffer to a bunch of individually specified colors.
    SetBufferToMany,
    /// Set the entire buffer to a single color.
    SetBufferToSingle,
    /// Set a single pixel index to a certain color.
    SetSinglePixel,

    /// Set the entire buffer to a bunch of individually specified colors, with alpha control.
    SetBufferToManyAlpha,
    /// Set the entire buffer to a specific color, with alpha control.
    SetBufferToSingleAlpha,
}

/// Describes the current state of a node.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum StripMode {
    /// LEDs are turned off.
    Off,
    /// LEDs use effect modes that render on the node.
    #[default]
    Effects,
    /// LEDs use streamed color data from the server.
    Dynamic,
    /// LEDs use effect modes, but streamed data can include an alpha channel
    /// that will dictate how much of streamed data versus effects data to use.
    Hybrid,
}
