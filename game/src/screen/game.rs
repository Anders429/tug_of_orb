use super::Screen;
use crate::{
    bios::wait_for_vblank,
    game::{self, Direction, Node, Position, Turn},
    include_bytes_aligned,
    mmio::{
        keys::KeyInput,
        vram::{
            BackgroundControl, BlendControl, ColorEffect, DisplayControl, ObjectAttributes,
            TextScreenEntry,
        },
        BG0CNT, BG1CNT, BG1HOFS, BG1VOFS, BG2CNT, BG2HOFS, BG2VOFS, BG_PALETTE, BLDCNT, BLDY,
        CHARBLOCK0, DISPCNT, KEYINPUT, OBJ_ATTRS, OBJ_PALETTE, OBJ_TILES, TEXT_SCREENBLOCK0,
        TEXT_SCREENBLOCK16, TEXT_SCREENBLOCK24,
    },
};
use core::{mem::transmute, ops::BitOrAssign};
use deranged::{RangedU16, RangedU8};
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
    ($file_name:literal, $offset:expr, $len:expr) => {
        unsafe {
            CHARBLOCK0
                .add($offset)
                .cast::<[[u32; 8]; $len]>()
                .write_volatile(transmute(include_bytes_aligned!($file_name).0));
        }
    };
}

/// Sets an individual screenblock.
///
/// This is basically just writing a single 8x8 tile.
fn set_block(x: usize, y: usize, tile: RangedU16<0, 1023>, frame: usize, palette: RangedU8<0, 15>) {
    unsafe {
        TEXT_SCREENBLOCK0
            .byte_add(frame * 0x800)
            .add(y * 32 + x)
            .write_volatile(TextScreenEntry::new().with_tile(tile).with_palette(palette));
    }
}

/// Set the tiles for an (x, y) position to a single tile.
///
/// This tile will be used four times, as an (x, y) position occupies four tile spaces.
fn set_tile(x: usize, y: usize, tile: RangedU16<0, 1023>, frame: usize, palette: RangedU8<0, 15>) {
    set_block(x * 2, y * 2, tile, frame, palette);
    set_block(x * 2 + 1, y * 2, tile, frame, palette);
    set_block(x * 2, y * 2 + 1, tile, frame, palette);
    set_block(x * 2 + 1, y * 2 + 1, tile, frame, palette);
}

/// Set the tiles for an (x, y) position to group of four sequential tiles.
fn set_tile_group(
    x: usize,
    y: usize,
    tile_start: RangedU16<0, 1023>,
    frame: usize,
    palette: RangedU8<0, 15>,
) {
    set_block(x * 2, y * 2, tile_start, frame, palette);
    set_block(
        x * 2 + 1,
        y * 2,
        tile_start.saturating_add(1),
        frame,
        palette,
    );
    set_block(
        x * 2,
        y * 2 + 1,
        tile_start.saturating_add(2),
        frame,
        palette,
    );
    set_block(
        x * 2 + 1,
        y * 2 + 1,
        tile_start.saturating_add(3),
        frame,
        palette,
    );
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

fn wait_frames(num: usize) {
    for _ in 0..num {
        wait_for_vblank();
    }
}

#[derive(Debug)]
struct ScrollAccelerator {
    position: (u16, u16),
}

impl ScrollAccelerator {
    fn new(position: Position) -> Self {
        Self {
            position: Self::position_to_pixel_location(position),
        }
    }

    fn position_to_pixel_location(position: Position) -> (u16, u16) {
        (position.x as u16 * 8 + 76, position.y as u16 * 12 + 86)
    }

    fn scroll_to_position(&mut self, position: Position, velocity: u16) -> bool {
        let target = Self::position_to_pixel_location(position);
        let x = if (self.position.0 > target.0) {
            if self.position.0 - target.0 >= velocity {
                self.position.0 - velocity
            } else {
                self.position.0 - 1
            }
        } else if (self.position.0 < target.0) {
            if target.0 - self.position.0 >= velocity {
                self.position.0 + velocity
            } else {
                self.position.0 + 1
            }
        } else {
            self.position.0
        };
        let y = if (self.position.1 > target.1) {
            if self.position.1 - target.1 >= velocity {
                self.position.1 - velocity
            } else {
                self.position.1 - 1
            }
        } else if (self.position.1 < target.1) {
            if target.1 - self.position.1 >= velocity {
                self.position.1 + velocity
            } else {
                self.position.1 + 1
            }
        } else {
            self.position.1
        };
        unsafe {
            BG1HOFS.write_volatile(RangedU16::new_unchecked(x));
            BG1VOFS.write_volatile(RangedU16::new_unchecked(y));
            BG2HOFS.write_volatile(RangedU16::new_unchecked(x));
            BG2VOFS.write_volatile(RangedU16::new_unchecked(y));
        }
        self.position = (x, y);
        target == self.position
    }

    fn relative_sprite_location(&self, position: Position) -> Option<(u16, u16)> {
        let target = (position.x as u16 * 8 + 52, position.y as u16 * 4 + 42);
        let top_left = Self::position_to_pixel_location(position);

        let x = {
            let (x, overflow) = target
                .0
                .overflowing_add_signed(top_left.0 as i16 - self.position.0 as i16);
            if x.wrapping_add(32) > 512 {
                return None;
            }
            x
        };
        let y = {
            let (y, overflow) = target
                .1
                .overflowing_add_signed(top_left.1 as i16 - self.position.1 as i16);
            if y.wrapping_add(32) > 256 {
                return None;
            }
            y
        };

        Some((x, y))
    }
}

#[derive(Debug)]
pub struct Game {
    cursor: Position,
    prev_keys: KeyInput,

    state: game::Game,
    player_color: game::Color,

    scroll_accelerator: ScrollAccelerator,
    scroll_at_start_of_player_turn: bool,
}

impl Game {
    pub fn new(cursor: Position, game: game::Game, player_color: game::Color) -> Self {
        wait_for_vblank();

        unsafe {
            // Initialize fade.
            BLDCNT.write_volatile(
                BlendControl::new()
                    .with_target1_bg0(true)
                    .with_target1_bg1(true)
                    .with_target1_bg2(true)
                    .with_target1_bg3(true)
                    .with_target1_obj(true)
                    .with_target1_backdrop(true)
                    .with_color_effect(ColorEffect::Brighten),
            );
            // Fade out while we set up the screen.
            BLDY.write_volatile(RangedU8::new_static::<16>());

            // Set up background layers.
            BG0CNT.write_volatile(
                BackgroundControl::new()
                    .with_screenblock(RangedU8::new_static::<8>())
                    .with_priority(RangedU8::new_static::<3>()),
            );
            BG1CNT.write_volatile(
                BackgroundControl::new()
                    .with_screenblock(RangedU8::new_static::<16>())
                    .with_priority(RangedU8::new_static::<2>())
                    .with_screen_size(RangedU8::new_static::<3>()),
            );
            BG2CNT.write_volatile(
                BackgroundControl::new()
                    .with_screenblock(RangedU8::new_static::<24>())
                    .with_priority(RangedU8::new_static::<1>())
                    .with_screen_size(RangedU8::new_static::<3>()),
            );
            DISPCNT.write_volatile(
                DisplayControl::new()
                    .with_bg0(true)
                    .with_bg1(true)
                    .with_bg2(true)
                    .with_obj(true)
                    .with_obj_vram_1d(true),
            );

            // Load palettes.
            BG_PALETTE.write_volatile(transmute(include_bytes_aligned!("../../res/neutral.pal").0));
            BG_PALETTE
                .add(1)
                .write_volatile(transmute(include_bytes_aligned!("../../res/red.pal").0));
            BG_PALETTE
                .add(2)
                .write_volatile(transmute(include_bytes_aligned!("../../res/blue.pal").0));
            BG_PALETTE
                .add(3)
                .write_volatile(transmute(include_bytes_aligned!("../../res/yellow.pal").0));
            BG_PALETTE
                .add(4)
                .write_volatile(transmute(include_bytes_aligned!("../../res/green.pal").0));
            OBJ_PALETTE.write_volatile(transmute(include_bytes_aligned!("../../res/cursor.pal").0));
        }

        // Define the game tiles.
        load_tiles!("../../res/empty.4bpp", 0, 1);
        load_tiles!("../../res/wall.4bpp", 1, 4);
        load_tiles!("../../res/arrow_right.4bpp", 5, 4);
        load_tiles!("../../res/arrow_left.4bpp", 9, 4);
        load_tiles!("../../res/arrow_down.4bpp", 13, 4);
        load_tiles!("../../res/arrow_up.4bpp", 17, 4);
        load_tiles!("../../res/grid0.4bpp", 21, 1);
        load_tiles!("../../res/grid0_left.4bpp", 22, 1);
        load_tiles!("../../res/grid0_up.4bpp", 23, 1);
        load_tiles!("../../res/grid0_left_up.4bpp", 24, 1);
        load_tiles!("../../res/grid1.4bpp", 25, 1);
        load_tiles!("../../res/grid1_right.4bpp", 26, 1);
        load_tiles!("../../res/grid1_up.4bpp", 27, 1);
        load_tiles!("../../res/grid1_right_up.4bpp", 28, 1);
        load_tiles!("../../res/grid2.4bpp", 29, 1);
        load_tiles!("../../res/grid2_left.4bpp", 30, 1);
        load_tiles!("../../res/grid2_down.4bpp", 31, 1);
        load_tiles!("../../res/grid2_left_down.4bpp", 32, 1);
        load_tiles!("../../res/grid3.4bpp", 33, 1);
        load_tiles!("../../res/grid3_right.4bpp", 34, 1);
        load_tiles!("../../res/grid3_down.4bpp", 35, 1);
        load_tiles!("../../res/grid3_right_down.4bpp", 36, 1);
        load_tiles!("../../res/background.4bpp", 37, 1);
        load_tiles!("../../res/arrow_all.4bpp", 38, 4);
        load_tiles!("../../res/super_arrow_left.4bpp", 42, 4);
        load_tiles!("../../res/super_arrow_up.4bpp", 46, 4);
        load_tiles!("../../res/super_arrow_right.4bpp", 50, 4);
        load_tiles!("../../res/super_arrow_down.4bpp", 54, 4);

        // Define the cursor tiles.
        unsafe {
            OBJ_TILES
                .cast::<[[u32; 8]; 4]>()
                .write_volatile(transmute::<_, [[u32; 8]; 4]>(
                    include_bytes_aligned!("../../res/cursor.4bpp").0,
                ))
        }

        // Draw background.
        for y in 0..16 {
            for x in 0..16 {
                set_tile(
                    x,
                    y,
                    RangedU16::new_static::<37>(),
                    8,
                    RangedU8::new_static::<1>(),
                );
            }
        }

        // Clear BGs.
        unsafe {
            TEXT_SCREENBLOCK16
                .cast::<[TextScreenEntry; 4096]>()
                .write_volatile(
                    [TextScreenEntry::new()
                        .with_tile(RangedU16::new_static::<0>())
                        .with_palette(RangedU8::new_static::<1>()); 4096],
                );
            TEXT_SCREENBLOCK24
                .cast::<[TextScreenEntry; 4096]>()
                .write_volatile(
                    [TextScreenEntry::new()
                        .with_tile(RangedU16::new_static::<0>())
                        .with_palette(RangedU8::new_static::<0>()); 4096],
                );
        }

        // Hide unused objects.
        unsafe {
            OBJ_ATTRS
                .add(1)
                .cast::<[ObjectAttributes; 127]>()
                .write_volatile([ObjectAttributes::new().with_disabled(true); 127])
        }

        let state = Self {
            cursor,
            prev_keys: KeyInput::NONE,

            state: game,
            player_color,

            scroll_accelerator: ScrollAccelerator::new(cursor),
            scroll_at_start_of_player_turn: false,
        };

        // Draw the initial game state.
        state.draw();

        // Draw the cursor.
        unsafe {
            OBJ_ATTRS.write_volatile(
                ObjectAttributes::new()
                    .with_x(state.cursor.x as u16 * 8 + 52)
                    .with_y(state.cursor.y as u8 * 4 + 42)
                    .with_tile(RangedU16::new_static::<0>())
                    .with_palette(RangedU8::new_static::<0>())
                    .with_size(RangedU8::new_static::<1>()),
            );
        }

        // Scroll.
        unsafe {
            BG1HOFS.write_volatile(RangedU16::new_unchecked(state.cursor.x as u16 * 8 + 76));
            BG1VOFS.write_volatile(RangedU16::new_unchecked(state.cursor.y as u16 * 12 + 86));
            BG2HOFS.write_volatile(RangedU16::new_unchecked(state.cursor.x as u16 * 8 + 76));
            BG2VOFS.write_volatile(RangedU16::new_unchecked(state.cursor.y as u16 * 12 + 86));
        }

        // Fade in.
        for fade in (0..31).rev() {
            wait_for_vblank();
            unsafe {
                BLDY.write_volatile(RangedU8::new_unchecked(fade / 2));
            }
        }

        state
    }

    fn draw(&self) {
        // Calculate node edges.
        let mut edges = [[Edges::new(); 16]; 16];
        for (y, row) in self.state.grid().iter().enumerate() {
            for (x, node) in row.iter().enumerate() {
                if !node.is_hidden() {
                    if let Some(direction) = node.direction() {
                        if (direction == Direction::Up && y == 0)
                            || (direction == Direction::Left && x == 0)
                            || (direction == Direction::Down && y == 15)
                            || (direction == Direction::Right && x == 15)
                        {
                            continue;
                        }
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
                    } else if node.all_directions() {
                        for direction in [
                            Direction::Left,
                            Direction::Up,
                            Direction::Right,
                            Direction::Down,
                        ] {
                            if (direction == Direction::Up && y == 0)
                                || (direction == Direction::Left && x == 0)
                                || (direction == Direction::Down && y == 15)
                                || (direction == Direction::Right && x == 15)
                            {
                                continue;
                            }
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
            }
        }

        for (y, row) in self.state.grid().iter().zip(edges).enumerate() {
            for (x, (node, edges)) in row.0.iter().zip(row.1).enumerate() {
                let (x, y, frame) = get_screen_location(x, y, 24);

                // Draw node.
                let palette = match node {
                    Node::Empty => {
                        set_tile(
                            x,
                            y,
                            RangedU16::new_static::<0>(),
                            frame,
                            RangedU8::new_static::<0>(),
                        );
                        RangedU8::new_static::<0>()
                    }
                    Node::Wall => {
                        set_tile_group(
                            x,
                            y,
                            RangedU16::new_static::<1>(),
                            frame,
                            RangedU8::new_static::<0>(),
                        );
                        RangedU8::new_static::<0>()
                    }
                    Node::Arrow {
                        direction,
                        alignment,
                    } => {
                        let palette = match alignment {
                            Some(game::Color::Red) => RangedU8::new_static::<1>(),
                            Some(game::Color::Blue) => RangedU8::new_static::<2>(),
                            Some(game::Color::Yellow) => RangedU8::new_static::<3>(),
                            Some(game::Color::Green) => RangedU8::new_static::<4>(),
                            _ => RangedU8::new_static::<0>(),
                        };
                        match direction {
                            Direction::Left => {
                                set_tile_group(x, y, RangedU16::new_static::<9>(), frame, palette);
                            }
                            Direction::Right => {
                                set_tile_group(x, y, RangedU16::new_static::<5>(), frame, palette);
                            }
                            Direction::Down => {
                                set_tile_group(x, y, RangedU16::new_static::<13>(), frame, palette);
                            }
                            Direction::Up => {
                                set_tile_group(x, y, RangedU16::new_static::<17>(), frame, palette);
                            }
                        }
                        palette
                    }
                    Node::AllDirection { alignment } => {
                        let palette = match alignment {
                            Some(game::Color::Red) => RangedU8::new_static::<1>(),
                            Some(game::Color::Blue) => RangedU8::new_static::<2>(),
                            Some(game::Color::Yellow) => RangedU8::new_static::<3>(),
                            Some(game::Color::Green) => RangedU8::new_static::<4>(),
                            _ => RangedU8::new_static::<0>(),
                        };
                        if alignment.is_some() {
                            set_tile_group(x, y, RangedU16::new_static::<38>(), frame, palette);
                        } else {
                            set_tile_group(x, y, RangedU16::new_static::<1>(), frame, palette);
                        }
                        palette
                    }
                    Node::SuperArrow {
                        alignment,
                        direction,
                    } => {
                        let palette = match alignment {
                            Some(game::Color::Red) => RangedU8::new_static::<1>(),
                            Some(game::Color::Blue) => RangedU8::new_static::<2>(),
                            Some(game::Color::Yellow) => RangedU8::new_static::<3>(),
                            Some(game::Color::Green) => RangedU8::new_static::<4>(),
                            _ => RangedU8::new_static::<0>(),
                        };
                        if alignment.is_some() {
                            match direction {
                                Direction::Left => {
                                    set_tile_group(
                                        x,
                                        y,
                                        RangedU16::new_static::<42>(),
                                        frame,
                                        palette,
                                    );
                                }
                                Direction::Right => {
                                    set_tile_group(
                                        x,
                                        y,
                                        RangedU16::new_static::<50>(),
                                        frame,
                                        palette,
                                    );
                                }
                                Direction::Down => {
                                    set_tile_group(
                                        x,
                                        y,
                                        RangedU16::new_static::<54>(),
                                        frame,
                                        palette,
                                    );
                                }
                                Direction::Up => {
                                    set_tile_group(
                                        x,
                                        y,
                                        RangedU16::new_static::<46>(),
                                        frame,
                                        palette,
                                    );
                                }
                            }
                        } else {
                            set_tile_group(x, y, RangedU16::new_static::<1>(), frame, palette);
                        }
                        palette
                    }
                };

                // Handle each corner of the edge tile separately.

                // Top left
                match (edges.contains(Edges::LEFT), edges.contains(Edges::UP)) {
                    (false, false) => set_block(
                        2 * x,
                        2 * y,
                        RangedU16::new_static::<21>(),
                        frame - 8,
                        palette,
                    ),
                    (true, false) => set_block(
                        2 * x,
                        2 * y,
                        RangedU16::new_static::<22>(),
                        frame - 8,
                        palette,
                    ),
                    (false, true) => set_block(
                        2 * x,
                        2 * y,
                        RangedU16::new_static::<23>(),
                        frame - 8,
                        palette,
                    ),
                    (true, true) => set_block(
                        2 * x,
                        2 * y,
                        RangedU16::new_static::<24>(),
                        frame - 8,
                        palette,
                    ),
                }
                // Top right
                match (edges.contains(Edges::RIGHT), edges.contains(Edges::UP)) {
                    (false, false) => set_block(
                        2 * x + 1,
                        2 * y,
                        RangedU16::new_static::<25>(),
                        frame - 8,
                        palette,
                    ),
                    (true, false) => set_block(
                        2 * x + 1,
                        2 * y,
                        RangedU16::new_static::<26>(),
                        frame - 8,
                        palette,
                    ),
                    (false, true) => set_block(
                        2 * x + 1,
                        2 * y,
                        RangedU16::new_static::<27>(),
                        frame - 8,
                        palette,
                    ),
                    (true, true) => set_block(
                        2 * x + 1,
                        2 * y,
                        RangedU16::new_static::<28>(),
                        frame - 8,
                        palette,
                    ),
                }
                // Bottom left
                match (edges.contains(Edges::LEFT), edges.contains(Edges::DOWN)) {
                    (false, false) => set_block(
                        2 * x,
                        2 * y + 1,
                        RangedU16::new_static::<29>(),
                        frame - 8,
                        palette,
                    ),
                    (true, false) => set_block(
                        2 * x,
                        2 * y + 1,
                        RangedU16::new_static::<30>(),
                        frame - 8,
                        palette,
                    ),
                    (false, true) => set_block(
                        2 * x,
                        2 * y + 1,
                        RangedU16::new_static::<31>(),
                        frame - 8,
                        palette,
                    ),
                    (true, true) => set_block(
                        2 * x,
                        2 * y + 1,
                        RangedU16::new_static::<32>(),
                        frame - 8,
                        palette,
                    ),
                }
                // Bottom right
                match (edges.contains(Edges::RIGHT), edges.contains(Edges::DOWN)) {
                    (false, false) => set_block(
                        2 * x + 1,
                        2 * y + 1,
                        RangedU16::new_static::<33>(),
                        frame - 8,
                        palette,
                    ),
                    (true, false) => set_block(
                        2 * x + 1,
                        2 * y + 1,
                        RangedU16::new_static::<34>(),
                        frame - 8,
                        palette,
                    ),
                    (false, true) => set_block(
                        2 * x + 1,
                        2 * y + 1,
                        RangedU16::new_static::<35>(),
                        frame - 8,
                        palette,
                    ),
                    (true, true) => set_block(
                        2 * x + 1,
                        2 * y + 1,
                        RangedU16::new_static::<36>(),
                        frame - 8,
                        palette,
                    ),
                }
            }
        }
    }

    pub fn run(&mut self) -> Option<Screen> {
        if self.state.is_eliminated(self.player_color) {
            return Some(Screen::GameOver(super::GameOver::new(
                super::game_over::PlayerResult::Lose,
            )));
        }
        if self.state.turn_color() == self.player_color {
            // Read keys for each frame.
            let keys = unsafe { KEYINPUT.read_volatile() };
            let mut state_changed = false;

            if keys.contains(KeyInput::START) && !self.prev_keys.contains(KeyInput::START) {
                log::info!("cursor: {:?}", self.cursor);
            }
            const MAX_POSITION: Position = Position { x: 15, y: 15 };
            if keys.contains(KeyInput::RIGHT) && !self.prev_keys.contains(KeyInput::RIGHT) {
                self.cursor = self.cursor.move_saturating(Direction::Right, MAX_POSITION);
            }
            if keys.contains(KeyInput::UP) && !self.prev_keys.contains(KeyInput::UP) {
                self.cursor = self.cursor.move_saturating(Direction::Up, MAX_POSITION);
            }
            if keys.contains(KeyInput::LEFT) && !self.prev_keys.contains(KeyInput::LEFT) {
                self.cursor = self.cursor.move_saturating(Direction::Left, MAX_POSITION);
            }
            if keys.contains(KeyInput::DOWN) && !self.prev_keys.contains(KeyInput::DOWN) {
                self.cursor = self.cursor.move_saturating(Direction::Down, MAX_POSITION);
            }
            if keys.contains(KeyInput::A) && !self.prev_keys.contains(KeyInput::A) {
                let result = self.state.execute_turn(Turn {
                    rotate: self.cursor,
                });
                if let Ok(winner) = result {
                    state_changed = true;
                    if winner.is_some() {
                        wait_for_vblank();

                        self.draw();

                        return Some(Screen::GameOver(super::GameOver::new(
                            super::game_over::PlayerResult::Win,
                        )));
                    }
                }
            }

            self.prev_keys = keys;

            wait_for_vblank();

            // Scroll.
            if self.scroll_at_start_of_player_turn {
                self.scroll_at_start_of_player_turn =
                    !self.scroll_accelerator.scroll_to_position(self.cursor, 2);
            } else {
                self.scroll_accelerator.scroll_to_position(self.cursor, 1);
            }

            // Draw the cursor.
            if let Some(obj_pixel_pos) = self
                .scroll_accelerator
                .relative_sprite_location(self.cursor)
            {
                unsafe {
                    OBJ_ATTRS.write_volatile(
                        ObjectAttributes::new()
                            .with_x(obj_pixel_pos.0)
                            .with_y(obj_pixel_pos.1 as u8)
                            .with_tile(RangedU16::new_static::<0>())
                            .with_palette(RangedU8::new_static::<0>())
                            .with_size(RangedU8::new_static::<1>()),
                    );
                }
            }

            if state_changed {
                self.draw();
            }
        } else {
            // Determine the best move.
            let mut best_position = None;
            let mut best_weight = None;
            for x in 0..16 {
                for y in 0..16 {
                    let node = self.state.grid().get(Position { x, y }).unwrap();
                    if node.is_color(self.state.turn_color()) {
                        if let Some(direction) = node.direction() {
                            if let Some(best_weight_inner) = best_weight {
                                if let Some(new_pos) =
                                    (Position { x, y }).r#move(direction.clockwise())
                                {
                                    if !self
                                        .state
                                        .grid()
                                        .get(new_pos)
                                        .unwrap()
                                        .is_color(self.state.turn_color())
                                    {
                                        let weight = self.state.weight(new_pos);
                                        if weight > best_weight_inner {
                                            best_weight = Some(weight);
                                            best_position = Some((x, y));
                                        }
                                    }
                                }
                            } else {
                                if let Some(new_pos) =
                                    (Position { x, y }).r#move(direction.clockwise())
                                {
                                    if !self
                                        .state
                                        .grid()
                                        .get(new_pos)
                                        .unwrap()
                                        .is_color(self.state.turn_color())
                                    {
                                        best_weight = Some(self.state.weight(new_pos));
                                    } else {
                                        best_weight = Some(0);
                                    }
                                } else {
                                    best_weight = Some(0);
                                }
                                best_position = Some((x, y));
                            }
                        }
                    }
                }
            }

            let (x, y) = best_position.unwrap();
            self.state
                .execute_turn(Turn {
                    rotate: Position { x, y },
                })
                .unwrap();
            wait_for_vblank();
            loop {
                wait_for_vblank();
                let completed = self
                    .scroll_accelerator
                    .scroll_to_position(Position { x, y }, 2);

                if let Some(obj_pixel_pos) = self
                    .scroll_accelerator
                    .relative_sprite_location(self.cursor)
                {
                    unsafe {
                        OBJ_ATTRS.write_volatile(
                            ObjectAttributes::new()
                                .with_x(obj_pixel_pos.0)
                                .with_y(obj_pixel_pos.1 as u8)
                                .with_tile(RangedU16::new_static::<0>())
                                .with_palette(RangedU8::new_static::<0>())
                                .with_size(RangedU8::new_static::<1>()),
                        );
                    }
                }

                if completed {
                    break;
                }
            }

            wait_for_vblank();
            self.draw();
            wait_frames(30);
            self.scroll_at_start_of_player_turn = true;
        }

        None
    }
}
