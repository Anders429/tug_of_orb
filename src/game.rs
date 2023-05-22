//! The actual gameplay.

#[derive(Clone, Copy, Debug)]
pub enum Color {
    Red,
    Blue,
    Yellow,
    Green,
}

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

/// The game state.
#[derive(Debug)]
pub struct Game {
    player_color: Color,
    turn_color: Color,

    grid: [[Square; 16]; 16],
}

impl Game {
    pub fn builder() -> Builder {
        Builder {
            player_color: Color::Red,
            turn_color: Color::Red,

            grid: [[Square::Empty; 16]; 16],
        }
    }
}

/// Helper for building game state.
///
/// Default values are set when this is constructed. They can be changed if desired.
#[derive(Debug)]
pub struct Builder {
    player_color: Color,
    turn_color: Color,

    grid: [[Square; 16]; 16],
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

    pub fn grid(mut self, grid: [[Square; 16]; 16]) -> Self {
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
