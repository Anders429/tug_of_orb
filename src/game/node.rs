use super::{Color, Direction};

#[derive(Clone, Copy, Debug)]
pub enum Node {
    Empty,
    Wall,
    Arrow {
        alignment: Option<Color>,
        direction: Direction,
    },
}

impl Node {
    pub fn color(&self) -> Option<Color> {
        match self {
            Self::Arrow { alignment, .. } => *alignment,
            _ => None,
        }
    }

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

    pub fn direction(&self) -> Option<Direction> {
        if let Node::Arrow { direction, .. } = self {
            Some(*direction)
        } else {
            None
        }
    }

    /// Returns whether the color was set.
    ///
    /// If the alignment was already `color`, then `false` is returned.
    pub fn set_color(&mut self, color: Color) -> bool {
        if let Node::Arrow { alignment, .. } = self {
            if matches!(alignment, Some(color)) {
                false
            } else {
                *alignment = Some(color);
                true
            }
        } else {
            false
        }
    }
}
