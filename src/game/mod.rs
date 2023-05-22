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

/// The game state.
#[derive(Debug)]
pub struct Game {
    /// The player's color.
    player_color: Color,
    /// The color of the current turn player.
    turn_color: Color,

    grid: Grid,
}

impl Game {
    pub fn builder() -> Builder {
        Builder {
            player_color: Color::Red,
            turn_color: Color::Red,

            grid: Grid::new([[Square::Empty; 16]; 16]),
        }
    }

    pub fn take_turn(&mut self, turn: Turn) -> Result<(), turn::Error> {
        self.grid
            .get(turn.rotate)
            .ok_or(turn::Error::InvalidRotationPosition)?;

        todo!()
    }
}

/// Helper for building game state.
///
/// Default values are set when this is constructed. They can be changed if desired.
#[derive(Debug)]
pub struct Builder {
    player_color: Color,
    turn_color: Color,

    grid: Grid,
}

impl Builder {
    pub fn player_color(mut self, player_color: Color) -> Self {
        self.player_color = player_color;
        self
    }

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
            player_color: self.player_color,
            turn_color: self.turn_color,

            grid: self.grid,
        }
    }
}
