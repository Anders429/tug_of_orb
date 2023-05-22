use super::Color;

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    fn rotate(&mut self) {
        match self {
            Self::Left => Self::Up,
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
        };
    }
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
    pub fn is_color(&self, color: Color) -> bool {
        match self {
            Self::Arrow { alignment, .. } => matches!(alignment, Some(color)),
            _ => false,
        }
    }

    pub fn rotate(&mut self) {
        match self {
            Self::Arrow { direction, .. } => direction.rotate(),
            _ => {}
        }
    }
}
