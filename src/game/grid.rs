use super::{position::Position, square::Square};

#[derive(Debug)]
pub struct Grid([[Square; 16]; 16]);

impl Grid {
    pub fn new(grid: [[Square; 16]; 16]) -> Self {
        Self(grid)
    }

    pub fn get(&self, position: Position) -> Option<&Square> {
        self.0.get(position.y as usize)?.get(position.x as usize)
    }
}
