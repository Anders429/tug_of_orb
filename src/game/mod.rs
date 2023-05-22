//! The actual gameplay.

mod direction;
mod grid;
mod node;
mod position;
mod turn;

use core::num::NonZeroU16;
use direction::Direction;
use grid::Grid;
use node::Node;
use position::Position;
use turn::Turn;

#[derive(Clone, Copy, Debug)]
pub enum Color {
    // Player colors.
    Red,
    Blue,
    Yellow,
    Green,
}

#[derive(Debug)]
pub struct ColorCounts {
    red: Option<NonZeroU16>,
    blue: Option<NonZeroU16>,
    yellow: Option<NonZeroU16>,
    green: Option<NonZeroU16>,
}

impl ColorCounts {
    fn change_color_counts(&mut self, increment: Color, decrement: Option<Color>) {
        match increment {
            Color::Red => match self.red.as_mut() {
                Some(count) => *count = count.checked_add(1).expect("red count overflowed"),
                None => self.red = Some(unsafe { NonZeroU16::new_unchecked(1) }),
            },

            _ => {}
        }
    }
}

pub enum Conclusion {
    Undecided,
    Winner(Color),
}

/// The game state.
#[derive(Debug)]
pub struct Game {
    /// Indicates whose turn it is.
    turn_color: Color,

    // These counts must invariantly match with the number of colors in `self.grid`.
    color_counts: ColorCounts,

    grid: Grid,
}

impl Game {
    pub fn builder() -> Builder {
        Builder {
            turn_color: Color::Red,

            grid: Grid::new([[Node::Empty; 16]; 16]),
        }
    }

    fn change_color_counts(&mut self, increment: Color, decrement: Option<Color>) {
        match increment {
            Color::Red => match self.color_counts.red.as_mut() {
                Some(count) => *count = count.checked_add(1).expect("red count overflowed"),
                None => self.color_counts.red = Some(unsafe { NonZeroU16::new_unchecked(1) }),
            },

            _ => {}
        }
    }

    /// Fill in the current color beginning at the given position.
    fn fill(&mut self, position: Position) {
        // Ensure this is a valid position.
        let node = match self.grid.get_mut(position) {
            Some(node) => node,
            None => return,
        };
        let old_color = node.color();
        if !node.set_color(self.turn_color) {
            // Stop if the color was not changed; that means we have already been this direction
            // before.
            return;
        }

        // Deal with the node this node points to.
        if let Some(direction) = node.direction() {
            if let Some(new_position) = position.r#move(direction) {
                self.fill(new_position);
            }
        }

        // Deal with the nodes pointing to this node.
        for direction in [
            Direction::Left,
            Direction::Up,
            Direction::Right,
            Direction::Down,
        ] {
            if let Some(new_position) = position.r#move(direction) {
                if let Some(new_node) = self.grid.get(new_position) {
                    if new_node.direction() == Some(direction.opposite()) {
                        self.fill(new_position);
                    }
                }
            }
        }
    }

    /// Make it the next player's turn.
    ///
    /// Returns false if the turn color was not changed.
    fn increment_turn(&mut self) -> bool {
        self.turn_color = match self.turn_color {
            Color::Red => {
                if self.color_counts.blue.is_some() {
                    Color::Blue
                } else if self.color_counts.yellow.is_some() {
                    Color::Yellow
                } else if self.color_counts.green.is_some() {
                    Color::Green
                } else {
                    return false;
                }
            }
            Color::Blue => {
                if self.color_counts.yellow.is_some() {
                    Color::Yellow
                } else if self.color_counts.green.is_some() {
                    Color::Green
                } else if self.color_counts.red.is_some() {
                    Color::Red
                } else {
                    return false;
                }
            }
            Color::Yellow => {
                if self.color_counts.green.is_some() {
                    Color::Green
                } else if self.color_counts.red.is_some() {
                    Color::Red
                } else if self.color_counts.blue.is_some() {
                    Color::Blue
                } else {
                    return false;
                }
            }
            Color::Green => {
                if self.color_counts.red.is_some() {
                    Color::Red
                } else if self.color_counts.blue.is_some() {
                    Color::Blue
                } else if self.color_counts.yellow.is_some() {
                    Color::Yellow
                } else {
                    return false;
                }
            }
        };
        true
    }

    /// Execute turn for the current player.
    pub fn execute_turn(&mut self, turn: Turn) -> Result<Conclusion, turn::Error> {
        let node = self
            .grid
            .get_mut(turn.rotate)
            .ok_or(turn::Error::InvalidRotationPosition)?;
        if !node.is_color(self.turn_color) {
            return Err(turn::Error::InvalidRotationPosition);
        }

        node.rotate();
        self.fill(turn.rotate);

        self.increment_turn();
        Ok(Conclusion::Undecided)
    }
}

/// Helper for building game state.
///
/// Default values are set when this is constructed. They can be changed if desired.
#[derive(Debug)]
pub struct Builder {
    turn_color: Color,

    grid: Grid,
}

impl Builder {
    pub fn turn_color(mut self, turn_color: Color) -> Self {
        self.turn_color = turn_color;
        self
    }

    pub fn grid(mut self, grid: Grid) -> Self {
        self.grid = grid;
        self
    }

    pub fn build(self) -> Game {
        let color_counts = self.grid.color_counts();

        Game {
            turn_color: self.turn_color,

            color_counts,

            grid: self.grid,
        }
    }
}
