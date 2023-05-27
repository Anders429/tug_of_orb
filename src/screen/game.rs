use super::Screen;
use crate::{
    game,
    game::{Direction, Node, Position, Turn},
};
use core::{ops::BitOrAssign, slice};
use gba::{
    bios::VBlankIntrWait,
    keys::KeyInput,
    mmio::{
        bg_palbank, obj_palbank, BG0CNT, BG1CNT, BG1HOFS, BG1VOFS, BG2CNT, BG2HOFS, BG2VOFS,
        BLDCNT, BLDY, CHARBLOCK0_4BPP, DISPCNT, KEYINPUT, OBJ_ATTR0, OBJ_ATTR_ALL, OBJ_TILES,
        TEXT_SCREENBLOCKS,
    },
    video::{
        obj::{ObjAttr, ObjAttr0, ObjDisplayStyle},
        BackgroundControl, BlendControl, Color, ColorEffectMode, DisplayControl, TextEntry,
    },
    Align4,
};

#[derive(Clone, Copy, Debug)]
struct Edges(u8);

impl Edges {
    const LEFT: Edges = Edges(0b0000_0001);
    const UP: Edges = Edges(0b0000_0010);
    const RIGHT: Edges = Edges(0b0000_0100);
    const DOWN: Edges = Edges(0b0000_1000);

    fn new() -> Self {
        Self(0)
    }

    fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl From<Direction> for Edges {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Left => Edges::LEFT,
            Direction::Up => Edges::UP,
            Direction::Right => Edges::RIGHT,
            Direction::Down => Edges::DOWN,
        }
    }
}

impl BitOrAssign for Edges {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

macro_rules! load_tiles {
    ($file_name:literal, $offset:expr) => {
        let aligned_bytes = Align4(*include_bytes!($file_name));
        let bytes = aligned_bytes.as_u32_slice();
        let len = bytes.len() / 8;
        let tiles = unsafe { slice::from_raw_parts(bytes.as_ptr() as *const [u32; 8], len) };
        CHARBLOCK0_4BPP
            .as_region()
            .sub_slice($offset..len + $offset)
            .write_from_slice(tiles);
    };
}

/// Sets an individual screenblock.
///
/// This is basically just writing a single 8x8 tile.
fn set_block(x: usize, y: usize, tile: u16, frame: usize, palette: u16) {
    TEXT_SCREENBLOCKS
        .get_frame(frame)
        .expect("invalid frame")
        .get_row(y)
        .expect("invalid row")
        .get(x)
        .expect("invalid column")
        .write(TextEntry::new().with_tile(tile).with_palbank(palette));
}

/// Set the tiles for an (x, y) position to a single tile.
///
/// This tile will be used four times, as an (x, y) position occupies four tile spaces.
fn set_tile(x: usize, y: usize, tile: u16, frame: usize, palette: u16) {
    set_block(x * 2, y * 2, tile, frame, palette);
    set_block(x * 2 + 1, y * 2, tile, frame, palette);
    set_block(x * 2, y * 2 + 1, tile, frame, palette);
    set_block(x * 2 + 1, y * 2 + 1, tile, frame, palette);
}

/// Set the tiles for an (x, y) position to group of four sequential tiles.
fn set_tile_group(x: usize, y: usize, tile_start: u16, frame: usize, palette: u16) {
    set_block(x * 2, y * 2, tile_start, frame, palette);
    set_block(x * 2 + 1, y * 2, tile_start + 1, frame, palette);
    set_block(x * 2, y * 2 + 1, tile_start + 2, frame, palette);
    set_block(x * 2 + 1, y * 2 + 1, tile_start + 3, frame, palette);
}

// Returns x, y, and frame.
fn get_screen_location(mut x: usize, mut y: usize, mut frame: usize) -> (usize, usize, usize) {
    x = x + 8;
    y = y + 8;
    if x >= 16 {
        x -= 16;
        frame += 1;
    }
    if y >= 16 {
        y -= 16;
        frame += 2;
    }
    (x, y, frame)
}

#[derive(Debug)]
pub struct Game {
    cursor: Position,
    prev_keys: KeyInput,

    state: game::Game,
    player_color: game::Color,
}

impl Game {
    pub fn new(cursor: Position, game: game::Game, player_color: game::Color) -> Self {
        VBlankIntrWait();

        // Initialize fade.
        BLDCNT.write(
            BlendControl::new()
                .with_target1_bg0(true)
                .with_target1_bg1(true)
                .with_target1_bg2(true)
                .with_target1_bg3(true)
                .with_target1_obj(true)
                .with_mode(ColorEffectMode::Brighten),
        );
        // Fade out while we set up the screen.
        BLDY.write(16);

        // Set up background layers.
        BG0CNT.write(
            BackgroundControl::new()
                .with_screenblock(8)
                .with_priority(3),
        );
        BG1CNT.write(
            BackgroundControl::new()
                .with_screenblock(16)
                .with_priority(2)
                .with_size(3),
        );
        BG2CNT.write(
            BackgroundControl::new()
                .with_screenblock(24)
                .with_priority(1)
                .with_size(3),
        );
        DISPCNT.write(
            DisplayControl::new()
                .with_show_bg0(true)
                .with_show_bg1(true)
                .with_show_bg2(true)
                .with_show_obj(true)
                .with_obj_vram_1d(true),
        );

        // Define the neutral palette.
        for (index, bytes) in Align4(*include_bytes!("../../res/neutral.pal"))
            .as_u16_slice()
            .iter()
            .enumerate()
        {
            bg_palbank(0).index(index).write(Color(*bytes));
        }
        // Define the red palette.
        for (index, bytes) in Align4(*include_bytes!("../../res/red.pal"))
            .as_u16_slice()
            .iter()
            .enumerate()
        {
            bg_palbank(1).index(index).write(Color(*bytes));
        }
        // Define the blue palette.
        for (index, bytes) in Align4(*include_bytes!("../../res/blue.pal"))
            .as_u16_slice()
            .iter()
            .enumerate()
        {
            bg_palbank(2).index(index).write(Color(*bytes));
        }
        // Define the yellow palette.
        for (index, bytes) in Align4(*include_bytes!("../../res/yellow.pal"))
            .as_u16_slice()
            .iter()
            .enumerate()
        {
            bg_palbank(3).index(index).write(Color(*bytes));
        }
        // Define the green palette.
        for (index, bytes) in Align4(*include_bytes!("../../res/green.pal"))
            .as_u16_slice()
            .iter()
            .enumerate()
        {
            bg_palbank(4).index(index).write(Color(*bytes));
        }

        // Define cursor palette.
        for (index, bytes) in Align4(*include_bytes!("../../res/cursor.pal"))
            .as_u16_slice()
            .iter()
            .enumerate()
        {
            obj_palbank(0).index(index).write(Color(*bytes));
        }

        // Define the game tiles.
        load_tiles!("../../res/wall.4bpp", 1);
        load_tiles!("../../res/arrow_right.4bpp", 5);
        load_tiles!("../../res/arrow_left.4bpp", 9);
        load_tiles!("../../res/arrow_down.4bpp", 13);
        load_tiles!("../../res/arrow_up.4bpp", 17);
        load_tiles!("../../res/grid0.4bpp", 21);
        load_tiles!("../../res/grid0_left.4bpp", 22);
        load_tiles!("../../res/grid0_up.4bpp", 23);
        load_tiles!("../../res/grid0_left_up.4bpp", 24);
        load_tiles!("../../res/grid1.4bpp", 25);
        load_tiles!("../../res/grid1_right.4bpp", 26);
        load_tiles!("../../res/grid1_up.4bpp", 27);
        load_tiles!("../../res/grid1_right_up.4bpp", 28);
        load_tiles!("../../res/grid2.4bpp", 29);
        load_tiles!("../../res/grid2_left.4bpp", 30);
        load_tiles!("../../res/grid2_down.4bpp", 31);
        load_tiles!("../../res/grid2_left_down.4bpp", 32);
        load_tiles!("../../res/grid3.4bpp", 33);
        load_tiles!("../../res/grid3_right.4bpp", 34);
        load_tiles!("../../res/grid3_down.4bpp", 35);
        load_tiles!("../../res/grid3_right_down.4bpp", 36);
        load_tiles!("../../res/background.4bpp", 37);

        // Define the cursor tiles.
        let aligned_bytes = Align4(*include_bytes!("../../res/cursor.4bpp"));
        let bytes = aligned_bytes.as_u32_slice();
        let len = bytes.len() / 8;
        let tiles = unsafe { slice::from_raw_parts(bytes.as_ptr() as *const [u32; 8], len) };
        OBJ_TILES
            .as_region()
            .sub_slice(..len)
            .write_from_slice(tiles);

        // Draw background.
        for y in 0..16 {
            for x in 0..16 {
                set_tile(x, y, 37, 8, 1);
            }
        }

        // Hide unused objects.
        OBJ_ATTR0.iter().skip(1).for_each(|address| {
            address.write(ObjAttr0::new().with_style(ObjDisplayStyle::NotDisplayed))
        });

        let state = Self {
            cursor,
            prev_keys: KeyInput::new(),

            state: game,
            player_color,
        };

        // Draw the initial game state.
        state.draw();

        // Draw the cursor.
        let mut obj = ObjAttr::new();
        obj.set_x(state.cursor.x as u16 * 8 + 52);
        obj.set_y(state.cursor.y as u16 * 4 + 42);
        obj.set_tile_id(0);
        obj.set_palbank(0);
        obj.1 = obj.1.with_size(1);
        OBJ_ATTR_ALL.get(0).unwrap().write(obj);

        // Scroll.
        BG1HOFS.write(state.cursor.x as u16 * 8 + 76);
        BG1VOFS.write(state.cursor.y as u16 * 12 + 86);
        BG2HOFS.write(state.cursor.x as u16 * 8 + 76);
        BG2VOFS.write(state.cursor.y as u16 * 12 + 86);

        // Fade in.
        for fade in (0..31).rev() {
            VBlankIntrWait();
            BLDY.write(fade / 2);
        }

        state
    }

    fn draw(&self) {
        // Calculate node edges.
        let mut edges = [[Edges::new(); 16]; 16];
        for (y, row) in self.state.grid().iter().enumerate() {
            for (x, node) in row.iter().enumerate() {
                if let Some(direction) = node.direction() {
                    edges[y][x] |= direction.into();
                    // Update the edges of the pointed-at node.
                    if let Some(position) = (Position {
                        x: x as u8,
                        y: y as u8,
                    })
                    .r#move(direction)
                    {
                        if let Some(other_node_edges) = edges
                            .get_mut(position.y as usize)
                            .map(|row| row.get_mut(position.x as usize))
                            .flatten()
                        {
                            *other_node_edges |= direction.opposite().into();
                        }
                    }
                }
            }
        }

        for (y, row) in self.state.grid().iter().zip(edges).enumerate() {
            for (x, (node, edges)) in row.0.iter().zip(row.1).enumerate() {
                let (x, y, frame) = get_screen_location(x, y, 24);

                // Draw node.
                let palette = match node {
                    Node::Empty => {
                        set_tile(x, y, 0, frame, 0);
                        0
                    }
                    Node::Wall => {
                        set_tile_group(x, y, 1, frame, 0);
                        0
                    }
                    Node::Arrow {
                        direction,
                        alignment,
                    } => {
                        let palette = match alignment {
                            Some(game::Color::Red) => 1,
                            Some(game::Color::Blue) => 2,
                            Some(game::Color::Yellow) => 3,
                            Some(game::Color::Green) => 4,
                            _ => 0,
                        };
                        match direction {
                            Direction::Left => {
                                set_tile_group(x, y, 9, frame, palette);
                            }
                            Direction::Right => {
                                set_tile_group(x, y, 5, frame, palette);
                            }
                            Direction::Down => {
                                set_tile_group(x, y, 13, frame, palette);
                            }
                            Direction::Up => {
                                set_tile_group(x, y, 17, frame, palette);
                            }
                        }
                        palette
                    }
                };

                // Handle each corner of the edge tile separately.

                // Top left
                match (edges.contains(Edges::LEFT), edges.contains(Edges::UP)) {
                    (false, false) => set_block(2 * x, 2 * y, 21, frame - 8, palette),
                    (true, false) => set_block(2 * x, 2 * y, 22, frame - 8, palette),
                    (false, true) => set_block(2 * x, 2 * y, 23, frame - 8, palette),
                    (true, true) => set_block(2 * x, 2 * y, 24, frame - 8, palette),
                }
                // Top right
                match (edges.contains(Edges::RIGHT), edges.contains(Edges::UP)) {
                    (false, false) => set_block(2 * x + 1, 2 * y, 25, frame - 8, palette),
                    (true, false) => set_block(2 * x + 1, 2 * y, 26, frame - 8, palette),
                    (false, true) => set_block(2 * x + 1, 2 * y, 27, frame - 8, palette),
                    (true, true) => set_block(2 * x + 1, 2 * y, 28, frame - 8, palette),
                }
                // Bottom left
                match (edges.contains(Edges::LEFT), edges.contains(Edges::DOWN)) {
                    (false, false) => set_block(2 * x, 2 * y + 1, 29, frame - 8, palette),
                    (true, false) => set_block(2 * x, 2 * y + 1, 30, frame - 8, palette),
                    (false, true) => set_block(2 * x, 2 * y + 1, 31, frame - 8, palette),
                    (true, true) => set_block(2 * x, 2 * y + 1, 32, frame - 8, palette),
                }
                // Bottom right
                match (edges.contains(Edges::RIGHT), edges.contains(Edges::DOWN)) {
                    (false, false) => set_block(2 * x + 1, 2 * y + 1, 33, frame - 8, palette),
                    (true, false) => set_block(2 * x + 1, 2 * y + 1, 34, frame - 8, palette),
                    (false, true) => set_block(2 * x + 1, 2 * y + 1, 35, frame - 8, palette),
                    (true, true) => set_block(2 * x + 1, 2 * y + 1, 36, frame - 8, palette),
                }
            }
        }
    }

    pub fn run(&mut self) -> Option<Screen> {
        if self.state.turn_color() == self.player_color {
            // Read keys for each frame.
            let keys = KEYINPUT.read();
            let mut state_changed = false;

            if keys.start() && !self.prev_keys.start() {
                log::info!("cursor: {:?}", self.cursor);
            }
            const MAX_POSITION: Position = Position { x: 15, y: 15 };
            if keys.right() && !self.prev_keys.right() {
                self.cursor = self.cursor.move_saturating(Direction::Right, MAX_POSITION);
            }
            if keys.up() && !self.prev_keys.up() {
                self.cursor = self.cursor.move_saturating(Direction::Up, MAX_POSITION);
            }
            if keys.left() && !self.prev_keys.left() {
                self.cursor = self.cursor.move_saturating(Direction::Left, MAX_POSITION);
            }
            if keys.down() && !self.prev_keys.down() {
                self.cursor = self.cursor.move_saturating(Direction::Down, MAX_POSITION);
            }
            if keys.a() && !self.prev_keys.a() {
                if self
                    .state
                    .execute_turn(Turn {
                        rotate: self.cursor,
                    })
                    .is_ok()
                {
                    state_changed = true;
                }
            }

            self.prev_keys = keys;

            VBlankIntrWait();

            // Draw the cursor.
            let mut obj = ObjAttr::new();
            obj.set_x(self.cursor.x as u16 * 8 + 52);
            obj.set_y(self.cursor.y as u16 * 4 + 42);
            obj.set_tile_id(0);
            obj.set_palbank(0);
            obj.1 = obj.1.with_size(1);
            OBJ_ATTR_ALL.get(0).unwrap().write(obj);

            if state_changed {
                self.draw();
            }

            // Scroll.
            BG1HOFS.write(self.cursor.x as u16 * 8 + 76);
            BG1VOFS.write(self.cursor.y as u16 * 12 + 86);
            BG2HOFS.write(self.cursor.x as u16 * 8 + 76);
            BG2VOFS.write(self.cursor.y as u16 * 12 + 86);
        } else {
            'outer: for x in 0..16 {
                for y in 0..16 {
                    if self
                        .state
                        .execute_turn(Turn {
                            rotate: Position { x, y },
                        })
                        .is_ok()
                    {
                        VBlankIntrWait();
                        self.draw();
                        break 'outer;
                    }
                }
            }
        }

        None
    }
}
