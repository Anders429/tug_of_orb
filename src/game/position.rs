use super::Direction;

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

impl Position {
    /// Attempt to move to a position one step away in the given direction.
    ///
    /// Will return `None` if no such position can be represented (i.e. it's out of bounds).
    pub fn r#move(self, direction: Direction) -> Option<Position> {
        match direction {
            Direction::Left => (self.x > 0).then(|| Position {
                x: self.x - 1,
                y: self.y,
            }),
            Direction::Up => (self.y > 0).then(|| Position {
                x: self.x,
                y: self.y - 1,
            }),
            Direction::Right => (self.x < u8::MAX).then(|| Position {
                x: self.x + 1,
                y: self.y,
            }),
            Direction::Down => (self.y < u8::MAX).then(|| Position {
                x: self.x,
                y: self.y + 1,
            }),
        }
    }

    pub fn move_saturating(self, direction: Direction, max: Position) -> Position {
        if let Some(new_position) = self.r#move(direction) {
            if new_position.x <= max.x && new_position.y <= max.y {
                new_position
            } else {
                self
            }
        } else {
            self
        }
    }
}
