use super::{Color, ColorCounts, Node, Position};
use core::{num::NonZeroU16, slice};

#[derive(Debug)]
pub struct Grid([[Node; 16]; 16]);

impl Grid {
    pub fn new(grid: [[Node; 16]; 16]) -> Self {
        Self(grid)
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
