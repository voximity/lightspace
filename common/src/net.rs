use num_enum::TryFromPrimitive;

/// A TCP message from the server to a node.
pub enum ServerMessage {}

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
}
