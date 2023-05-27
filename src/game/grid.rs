use super::{Color, ColorCounts, Direction, Node, Position};
use core::{num::NonZeroU16, slice};
use gba::random::{Gen32, Lcg32};

#[derive(Debug)]
pub struct Grid([[Node; 16]; 16]);

impl Grid {
    pub fn new(grid: [[Node; 16]; 16]) -> Self {
        Self(grid)
    }

    fn populate_reflected_arrows(&mut self, x: usize, y: usize, direction: Direction) {
        self.0[y][x] = Node::Arrow {
            alignment: None,
            direction,
        };
        self.0[15 - x][y] = Node::Arrow {
            alignment: None,
            direction: direction.counter_clockwise(),
        };
        self.0[x][15 - y] = Node::Arrow {
            alignment: None,
            direction: direction.clockwise(),
        };
        self.0[15 - y][15 - x] = Node::Arrow {
            alignment: None,
            direction: direction.opposite(),
        };
    }

    /// Generate a random grid.
    pub fn generate(seed: u32) -> Self {
        let mut grid = Grid([[Node::Empty; 16]; 16]);

        // Starting positions.
        grid.0[0][0] = Node::Arrow {
            alignment: Some(Color::Red),
            direction: Direction::Up,
        };
        grid.0[0][15] = Node::Arrow {
            alignment: Some(Color::Blue),
            direction: Direction::Right,
        };
        grid.0[15][0] = Node::Arrow {
            alignment: Some(Color::Yellow),
            direction: Direction::Left,
        };
        grid.0[15][15] = Node::Arrow {
            alignment: Some(Color::Green),
            direction: Direction::Down,
        };

        let mut lcg = Lcg32::new(seed);
        for y in 0..8 {
            for x in 0..8 {
                // Already did the starting positions.
                if x == 0 && y == 0 {
                    continue;
                }
                let rand = lcg.next_u8();
                lcg.jump_state(1);
                match rand {
                    0..=60 => {
                        if x == 1 && y == 0 {
                            grid.populate_reflected_arrows(x, y, Direction::Up);
                        } else {
                            grid.populate_reflected_arrows(x, y, Direction::Left);
                        }
                    }
                    61..=120 => {
                        if x == 0 && y == 1 {
                            grid.populate_reflected_arrows(x, y, Direction::Left);
                        } else {
                            grid.populate_reflected_arrows(x, y, Direction::Up);
                        }
                    }
                    121..=180 => {
                        grid.populate_reflected_arrows(x, y, Direction::Right);
                    }
                    181..=240 => {
                        grid.populate_reflected_arrows(x, y, Direction::Down);
                    }
                    241..=255 => {
                        if x == 0 {
                            grid.populate_reflected_arrows(x, y, Direction::Down)
                        } else if y == 0 {
                            grid.populate_reflected_arrows(x, y, Direction::Right)
                        } else {
                            grid.0[y][x] = Node::Wall;
                            grid.0[x][15 - y] = Node::Wall;
                            grid.0[15 - x][y] = Node::Wall;
                            grid.0[15 - y][15 - x] = Node::Wall;
                        }
                    }
                    _ => {}
                }
            }
        }

        grid
    }

    pub fn get(&self, position: Position) -> Option<&Node> {
        self.0.get(position.y as usize)?.get(position.x as usize)
    }

    pub fn get_mut(&mut self, position: Position) -> Option<&mut Node> {
        self.0
            .get_mut(position.y as usize)?
            .get_mut(position.x as usize)
    }

    pub fn color_counts(&self) -> ColorCounts {
        let mut red_count = 0;
        let mut blue_count = 0;
        let mut yellow_count = 0;
        let mut green_count = 0;

        for row in self.0 {
            for node in row {
                match node.color() {
                    Some(Color::Red) => red_count += 1,
                    Some(Color::Blue) => blue_count += 1,
                    Some(Color::Yellow) => yellow_count += 1,
                    Some(Color::Green) => green_count += 1,
                    None => {}
                }
            }
        }

        ColorCounts {
            red: red_count.try_into().ok(),
            blue: blue_count.try_into().ok(),
            yellow: yellow_count.try_into().ok(),
            green: green_count.try_into().ok(),
        }
    }

    pub fn iter(&self) -> slice::Iter<[Node; 16]> {
        self.0.iter()
    }
}
