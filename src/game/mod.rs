//! The actual gameplay.

mod grid;
mod position;
mod square;
mod turn;

use grid::Grid;
use square::Square;
use turn::Turn;

#[derive(Clone, Copy, Debug)]
pub enum Color {
    // Player colors.
    Red,
    Blue,
    Yellow,
    Green,
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

    grid: Grid,
}

impl Game {
    pub fn builder() -> Builder {
        Builder {
            turn_color: Color::Red,

            grid: Grid::new([[Square::Empty; 16]; 16]),
        }
    }

    fn increment_turn(&mut self) {
        self.turn_color = match self.turn_color {
            Color::Red => Color::Blue,
            Color::Blue => Color::Yellow,
            Color::Yellow => Color::Green,
            Color::Green => Color::Red,
        };
    }

    pub fn take_turn(&mut self, turn: Turn) -> Result<Conclusion, turn::Error> {
        let square = self
            .grid
            .get_mut(turn.rotate)
            .ok_or(turn::Error::InvalidRotationPosition)?;
        if !square.is_color(self.turn_color) {
            return Err(turn::Error::InvalidRotationPosition);
        }

        square.rotate();

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
        Game {
            turn_color: self.turn_color,

            grid: self.grid,
        }
    }
}
