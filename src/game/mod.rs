//! The actual gameplay.

mod direction;
mod grid;
mod node;
mod position;
mod turn;

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

            grid: Grid::new([[Node::Empty; 16]; 16]),
        }
    }

    /// Fill in the current color beginning at the given position.
    fn fill(&mut self, position: Position) {
        // Ensure this is a valid position.
        let node = match self.grid.get_mut(position) {
            Some(node) => node,
            None => return,
        };
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
    fn increment_turn(&mut self) {
        // TODO: Account for colors that no longer exist on the grid.
        self.turn_color = match self.turn_color {
            Color::Red => Color::Blue,
            Color::Blue => Color::Yellow,
            Color::Yellow => Color::Green,
            Color::Green => Color::Red,
        };
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
        Game {
            turn_color: self.turn_color,

            grid: self.grid,
        }
    }
}
