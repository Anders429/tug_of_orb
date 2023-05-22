use super::{Node, Position};

#[derive(Debug)]
pub struct Grid([[Node; 16]; 16]);

impl Grid {
    pub fn new(grid: [[Node; 16]; 16]) -> Self {
        Self(grid)
    }

    pub fn get(&self, position: Position) -> Option<&Node> {
        self.0.get(position.y as usize)?.get(position.x as usize)
    }

    pub fn get_mut(&mut self, position: Position) -> Option<&mut Node> {
        self.0
            .get_mut(position.y as usize)?
            .get_mut(position.x as usize)
    }
}
