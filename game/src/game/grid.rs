use super::{Color, ColorCounts, Direction, Node, Position};
use crate::random::Pcg32Fast;
use core::slice;
use rand::Rng;

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

    fn populate_wall(&mut self, x: usize, y: usize, pcg: &mut Pcg32Fast) {
        match pcg.gen::<u8>() {
            0..=63 => self.0[y][x] = Node::AllDirection { alignment: None },
            64..=127 => {
                self.0[y][x] = Node::SuperArrow {
                    alignment: None,
                    direction: {
                        match pcg.gen::<u8>() {
                            0..=63 => Direction::Left,
                            64..=127 => Direction::Up,
                            128..=191 => Direction::Right,
                            192..=255 => Direction::Down,
                        }
                    },
                }
            }
            64..=255 => self.0[y][x] = Node::Wall,
        }
    }

    /// Generate a random grid.
    pub fn generate(seed: u64) -> Self {
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

        let mut pcg = Pcg32Fast::new(seed);
        for y in 0..8 {
            for x in 0..8 {
                // Already did the starting positions.
                if x == 0 && y == 0 {
                    continue;
                }
                match pcg.gen::<u8>() {
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
                            grid.populate_wall(x, y, &mut pcg);
                            grid.populate_wall(15 - y, x, &mut pcg);
                            grid.populate_wall(y, 15 - x, &mut pcg);
                            grid.populate_wall(15 - x, 15 - y, &mut pcg);
                        }
                    }
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

    pub fn weight(&self, position: Position, visited: &mut [[bool; 16]; 16]) -> u8 {
        log::info!("position: {:?}", position);
        if visited[position.y as usize][position.x as usize] {
            0
        } else {
            visited[position.y as usize][position.x as usize] = true;
            if let Some(node) = self.get(position) {
                if node.is_hidden() {
                    0
                } else {
                    if let Some(direction) = node.direction() {
                        if let Some(new_position) = position.r#move(direction) {
                            1 + self.weight(new_position, visited)
                        } else {
                            1
                        }
                    } else if node.all_directions() {
                        let mut weight = 1;
                        for direction in [
                            Direction::Left,
                            Direction::Up,
                            Direction::Right,
                            Direction::Down,
                        ] {
                            if let Some(new_position) = position.r#move(direction) {
                                weight += self.weight(new_position, visited);
                            }
                        }
                        weight
                    } else {
                        0
                    }
                }
            } else {
                0
            }
        }
    }
}
