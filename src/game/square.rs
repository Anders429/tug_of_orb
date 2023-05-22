use super::Color;

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

#[derive(Clone, Copy, Debug)]
pub enum Square {
    Empty,
    Wall,
    Arrow {
        alignment: Option<Color>,
        direction: Direction,
    },
}

impl Square {
    pub fn is_rotatable(&self, color: Color) -> bool {
        match self {
            Self::Empty | Self::Wall => false,
            Self::Arrow { alignment, .. } => matches!(alignment, Some(color)),
        }
    }
}
