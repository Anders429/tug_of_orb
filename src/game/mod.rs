//! The actual gameplay.

mod direction;
mod grid;
mod node;
mod position;
mod turn;

pub use direction::Direction;
pub use grid::Grid;
pub use node::Node;
pub use position::Position;
pub use turn::Turn;

use core::num::NonZeroU16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    fn change(&mut self, increment: Color, decrement: Option<Color>) {
        match increment {
            Color::Red => match self.red.as_mut() {
                Some(count) => *count = count.checked_add(1).expect("red count overflowed"),
                None => self.red = Some(unsafe { NonZeroU16::new_unchecked(1) }),
            },
            Color::Blue => match self.blue.as_mut() {
                Some(count) => *count = count.checked_add(1).expect("blue count overflowed"),
                None => self.blue = Some(unsafe { NonZeroU16::new_unchecked(1) }),
            },
            Color::Yellow => match self.yellow.as_mut() {
                Some(count) => *count = count.checked_add(1).expect("yellow count overflowed"),
                None => self.yellow = Some(unsafe { NonZeroU16::new_unchecked(1) }),
            },
            Color::Green => match self.green.as_mut() {
                Some(count) => *count = count.checked_add(1).expect("green count overflowed"),
                None => self.green = Some(unsafe { NonZeroU16::new_unchecked(1) }),
            },
        }

        match decrement {
            Some(Color::Red) => {
                self.red = NonZeroU16::new(self.red.expect("red count underflowed").get() - 1)
            }
            Some(Color::Blue) => {
                self.blue = NonZeroU16::new(self.blue.expect("blue count underflowed").get() - 1)
            }
            Some(Color::Yellow) => {
                self.yellow =
                    NonZeroU16::new(self.yellow.expect("yellow count underflowed").get() - 1)
            }
            Some(Color::Green) => {
                self.green = NonZeroU16::new(self.green.expect("green count underflowed").get() - 1)
            }
            None => {}
        }
    }
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

    pub fn is_eliminated(&self, color: Color) -> bool {
        match color {
            Color::Red => self.color_counts.red.is_none(),
            Color::Blue => self.color_counts.blue.is_none(),
            Color::Yellow => self.color_counts.yellow.is_none(),
            Color::Green => self.color_counts.green.is_none(),
        }
    }

    /// Fill in the current color beginning at the given position.
    fn fill(&mut self, position: Position, visited: &mut [[bool; 16]; 16]) {
        // Ensure this is a valid position.
        let node = match self.grid.get_mut(position) {
            Some(node) => node,
            None => return,
        };

        if visited[position.y as usize][position.x as usize] {
            // We have already visited this position.
            return;
        }
        visited[position.y as usize][position.x as usize] = true;
        let old_color = node.color();
        if node.set_color(self.turn_color) {
            self.color_counts.change(self.turn_color, old_color);
        } else if !node.is_color(self.turn_color) {
            // This means it's a wall.
            return;
        }

        // Deal with the node this node points to.
        if !node.is_hidden() {
            if let Some(direction) = node.direction() {
                if let Some(new_position) = position.r#move(direction) {
                    self.fill(new_position, visited);
                }
            } else if node.all_directions() {
                for direction in [
                    Direction::Left,
                    Direction::Up,
                    Direction::Right,
                    Direction::Down,
                ] {
                    if let Some(new_position) = position.r#move(direction) {
                        self.fill(new_position, visited);
                    }
                }
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
                    if !new_node.is_hidden() {
                        if new_node.direction() == Some(direction.opposite())
                            || new_node.all_directions()
                        {
                            self.fill(new_position, visited);
                        }
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
    pub fn execute_turn(&mut self, turn: Turn) -> Result<Option<Color>, turn::Error> {
        let node = self
            .grid
            .get_mut(turn.rotate)
            .ok_or(turn::Error::InvalidRotationPosition)?;
        if !node.is_color(self.turn_color) {
            return Err(turn::Error::InvalidRotationPosition);
        }

        node.rotate();

        if let Node::SuperArrow { direction, .. } = node {
            let direction = *direction;
            let mut position = turn.rotate;
            while let Some(new_pos) = position.r#move(direction) {
                let node = self.grid.get_mut(new_pos).unwrap();
                if node.is_wall() {
                    break;
                }
                node.set_direction(direction);
                position = new_pos;
            }
        }

        self.fill(turn.rotate, &mut [[false; 16]; 16]);

        self.increment_turn();

        match (
            self.color_counts.red.is_some(),
            self.color_counts.blue.is_some(),
            self.color_counts.yellow.is_some(),
            self.color_counts.green.is_some(),
        ) {
            (true, false, false, false) => Ok(Some(Color::Red)),
            (false, true, false, false) => Ok(Some(Color::Blue)),
            (false, false, true, false) => Ok(Some(Color::Yellow)),
            (false, false, false, true) => Ok(Some(Color::Green)),
            _ => Ok(None),
        }
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn turn_color(&self) -> Color {
        self.turn_color
    }

    pub fn weight(&self, position: Position) -> u8 {
        self.grid.weight(position, &mut [[false; 16]; 16])
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
