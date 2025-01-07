use super::{Color, Direction};

#[derive(Clone, Copy, Debug)]
pub enum Node {
    Empty,
    Wall,
    Arrow {
        alignment: Option<Color>,
        direction: Direction,
    },
    // "Secret" nodes.
    AllDirection {
        alignment: Option<Color>,
    },
    SuperArrow {
        alignment: Option<Color>,
        direction: Direction,
    },
}

impl Node {
    pub fn color(&self) -> Option<Color> {
        match self {
            Self::Arrow { alignment, .. }
            | Self::AllDirection { alignment }
            | Self::SuperArrow { alignment, .. } => *alignment,
            _ => None,
        }
    }

    pub fn is_color(&self, color: Color) -> bool {
        match self {
            Self::Arrow { alignment, .. }
            | Self::AllDirection { alignment }
            | Self::SuperArrow { alignment, .. } => *alignment == Some(color),
            _ => false,
        }
    }

    pub fn rotate(&mut self) {
        match self {
            Self::Arrow { direction, .. } | Self::SuperArrow { direction, .. } => {
                direction.rotate()
            }
            _ => {}
        }
    }

    pub fn set_direction(&mut self, new_direction: Direction) {
        match self {
            Self::Arrow { direction, .. } | Self::SuperArrow { direction, .. } => {
                *direction = new_direction;
            }
            _ => {}
        }
    }

    pub fn direction(&self) -> Option<Direction> {
        if let Node::Arrow { direction, .. }
        | Self::SuperArrow { direction, .. }
        | Self::SuperArrow { direction, .. } = self
        {
            Some(*direction)
        } else {
            None
        }
    }

    pub fn all_directions(&self) -> bool {
        matches!(self, Self::AllDirection { .. })
    }

    /// Returns whether the color was set.
    ///
    /// If the alignment was already `color`, then `false` is returned.
    pub fn set_color(&mut self, color: Color) -> bool {
        if let Node::Arrow { alignment, .. }
        | Self::AllDirection { alignment }
        | Self::SuperArrow { alignment, .. } = self
        {
            if *alignment == Some(color) {
                false
            } else {
                *alignment = Some(color);
                true
            }
        } else {
            false
        }
    }

    pub fn is_hidden(&self) -> bool {
        match self {
            Node::AllDirection { alignment } | Self::SuperArrow { alignment, .. } => {
                alignment.is_none()
            }
            _ => false,
        }
    }

    pub fn is_wall(&self) -> bool {
        matches!(self, Self::Wall)
    }
}
