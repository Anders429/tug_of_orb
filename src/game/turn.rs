use super::position::Position;

#[derive(Debug)]
pub struct Turn {
    /// The position to be rotated.
    pub rotate: Position,
}

#[derive(Debug)]
pub enum Error {
    InvalidRotationPosition,
}
