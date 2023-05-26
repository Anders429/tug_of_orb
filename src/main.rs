#![no_std]
#![no_main]

mod game;

use core::{ops::BitOrAssign, slice};
use game::{Direction, Game, Grid, Node, Position, Turn};
use gba::{
    bios::VBlankIntrWait,
    interrupts::IrqBits,
    keys::KeyInput,
    mmio::{
        bg_palbank, obj_palbank, BG0CNT, BG1CNT, BG2CNT, CHARBLOCK0_4BPP, DISPCNT, DISPSTAT, IE,
        IME, KEYINPUT, OBJ_ATTR0, OBJ_ATTR_ALL, OBJ_TILES, TEXT_SCREENBLOCKS,
    },
    video::{
        obj::{ObjAttr, ObjAttr0, ObjDisplayStyle},
        BackgroundControl, Color, DisplayControl, DisplayStatus, TextEntry, VideoMode,
    },
    Align4,
};
#[cfg(debug_assertions)]
use ::{
    core::fmt::Write,
    log::error,
    mgba_log::{MgbaLogLevel, MgbaWriter},
};

/// This panic handler is specifically for debug mode.
///
/// When panicking in debug builds, the panic info is logged as an error. Following this, a fatal
/// log will occur to halt emulation and display an error to the user. This is done to ensure the
/// entirety of the panic info is displayed to the user, as mGBA only allows for up to 256 bytes to
/// be logged at once.
#[cfg(debug_assertions)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("{}", info);
    MgbaWriter::new(MgbaLogLevel::Fatal)
        .write_str("Halting due to panic. See logs for `PanicInfo`.");
    loop {}
}

/// This panic handler is specifically for release mode.
///
/// In release builds, panicking just causes the game to lock up. Ideally, panicking should not
/// occur at all in release builds.
#[cfg(not(debug_assertions))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

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

#[derive(Debug)]
struct State {
    cursor: game::Position,

    game: Game,
}

impl State {
    fn draw(&self) {
        // Calculate node edges.
        let mut edges = [[Edges::new(); 16]; 16];
        for (y, row) in self.game.grid().iter().enumerate() {
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

        for (y, row) in self.game.grid().iter().zip(edges).enumerate() {
            for (x, (node, edges)) in row.0.iter().zip(row.1).enumerate() {
                // Draw node.
                let palette = match node {
                    Node::Empty => {
                        set_tile(x, y, 0, 24, 0);
                        0
                    }
                    Node::Wall => {
                        set_tile_group(x, y, 1, 24, 0);
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
                                set_tile_group(x, y, 9, 24, palette);
                            }
                            Direction::Right => {
                                set_tile_group(x, y, 5, 24, palette);
                            }
                            Direction::Down => {
                                set_tile_group(x, y, 13, 24, palette);
                            }
                            Direction::Up => {
                                set_tile_group(x, y, 17, 24, palette);
                            }
                            _ => {}
                        }
                        palette
                    }
                    _ => 0,
                };

                // Handle each corner of the edge tile separately.

                // Top left
                match (edges.contains(Edges::LEFT), edges.contains(Edges::UP)) {
                    (false, false) => set_block(2 * x, 2 * y, 21, 16, palette),
                    (true, false) => set_block(2 * x, 2 * y, 22, 16, palette),
                    (false, true) => set_block(2 * x, 2 * y, 23, 16, palette),
                    (true, true) => set_block(2 * x, 2 * y, 24, 16, palette),
                }
                // Top right
                match (edges.contains(Edges::RIGHT), edges.contains(Edges::UP)) {
                    (false, false) => set_block(2 * x + 1, 2 * y, 25, 16, palette),
                    (true, false) => set_block(2 * x + 1, 2 * y, 26, 16, palette),
                    (false, true) => set_block(2 * x + 1, 2 * y, 27, 16, palette),
                    (true, true) => set_block(2 * x + 1, 2 * y, 28, 16, palette),
                }
                // Bottom left
                match (edges.contains(Edges::LEFT), edges.contains(Edges::DOWN)) {
                    (false, false) => set_block(2 * x, 2 * y + 1, 29, 16, palette),
                    (true, false) => set_block(2 * x, 2 * y + 1, 30, 16, palette),
                    (false, true) => set_block(2 * x, 2 * y + 1, 31, 16, palette),
                    (true, true) => set_block(2 * x, 2 * y + 1, 32, 16, palette),
                }
                // Bottom right
                match (edges.contains(Edges::RIGHT), edges.contains(Edges::DOWN)) {
                    (false, false) => set_block(2 * x + 1, 2 * y + 1, 33, 16, palette),
                    (true, false) => set_block(2 * x + 1, 2 * y + 1, 34, 16, palette),
                    (false, true) => set_block(2 * x + 1, 2 * y + 1, 35, 16, palette),
                    (true, true) => set_block(2 * x + 1, 2 * y + 1, 36, 16, palette),
                }
            }
        }
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

/// Entry point for the game.
#[no_mangle]
extern "C" fn main() -> ! {
    // Initialize the global logger.
    //
    // This logging only works in mGBA. It is only enabled in debug builds.
    #[cfg(debug_assertions)]
    mgba_log::init().expect("failed to initialize mgba logging");

    // Enable vblank interrupts.
    DISPSTAT.write(DisplayStatus::new().with_irq_vblank(true));
    IE.write(IrqBits::VBLANK);
    // Enable interrupts generally.
    IME.write(true);

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

    let mut state = State {
        game: Game::builder()
            .grid(Grid::new({
                let mut grid = [[Node::Arrow {
                    direction: Direction::Right,
                    alignment: None,
                }; 16]; 16];
                grid[0][0] = Node::Arrow {
                    direction: Direction::Up,
                    alignment: Some(game::Color::Red),
                };
                grid[0][1] = Node::Arrow {
                    direction: Direction::Up,
                    alignment: Some(game::Color::Blue),
                };
                grid[0][2] = Node::Arrow {
                    direction: Direction::Up,
                    alignment: Some(game::Color::Yellow),
                };
                grid[0][3] = Node::Arrow {
                    direction: Direction::Up,
                    alignment: Some(game::Color::Green),
                };
                grid
            }))
            .build(),

        cursor: Position { x: 0, y: 0 },
    };

    // Define the neutral palette.
    for (index, bytes) in Align4(*include_bytes!("../res/neutral.pal"))
        .as_u16_slice()
        .iter()
        .enumerate()
    {
        bg_palbank(0).index(index).write(Color(*bytes));
    }
    // Define the red palette.
    for (index, bytes) in Align4(*include_bytes!("../res/red.pal"))
        .as_u16_slice()
        .iter()
        .enumerate()
    {
        bg_palbank(1).index(index).write(Color(*bytes));
    }
    // Define the blue palette.
    for (index, bytes) in Align4(*include_bytes!("../res/blue.pal"))
        .as_u16_slice()
        .iter()
        .enumerate()
    {
        bg_palbank(2).index(index).write(Color(*bytes));
    }
    // Define the yellow palette.
    for (index, bytes) in Align4(*include_bytes!("../res/yellow.pal"))
        .as_u16_slice()
        .iter()
        .enumerate()
    {
        bg_palbank(3).index(index).write(Color(*bytes));
    }
    // Define the green palette.
    for (index, bytes) in Align4(*include_bytes!("../res/green.pal"))
        .as_u16_slice()
        .iter()
        .enumerate()
    {
        bg_palbank(4).index(index).write(Color(*bytes));
    }

    // Define cursor palette.
    for (index, bytes) in Align4(*include_bytes!("../res/cursor.pal"))
        .as_u16_slice()
        .iter()
        .enumerate()
    {
        obj_palbank(0).index(index).write(Color(*bytes));
    }

    // Define the game tiles.
    load_tiles!("../res/wall.4bpp", 1);
    load_tiles!("../res/arrow_right.4bpp", 5);
    load_tiles!("../res/arrow_left.4bpp", 9);
    load_tiles!("../res/arrow_down.4bpp", 13);
    load_tiles!("../res/arrow_up.4bpp", 17);
    load_tiles!("../res/grid0.4bpp", 21);
    load_tiles!("../res/grid0_left.4bpp", 22);
    load_tiles!("../res/grid0_up.4bpp", 23);
    load_tiles!("../res/grid0_left_up.4bpp", 24);
    load_tiles!("../res/grid1.4bpp", 25);
    load_tiles!("../res/grid1_right.4bpp", 26);
    load_tiles!("../res/grid1_up.4bpp", 27);
    load_tiles!("../res/grid1_right_up.4bpp", 28);
    load_tiles!("../res/grid2.4bpp", 29);
    load_tiles!("../res/grid2_left.4bpp", 30);
    load_tiles!("../res/grid2_down.4bpp", 31);
    load_tiles!("../res/grid2_left_down.4bpp", 32);
    load_tiles!("../res/grid3.4bpp", 33);
    load_tiles!("../res/grid3_right.4bpp", 34);
    load_tiles!("../res/grid3_down.4bpp", 35);
    load_tiles!("../res/grid3_right_down.4bpp", 36);
    load_tiles!("../res/background.4bpp", 37);

    // Define the cursor tiles.
    let aligned_bytes = Align4(*include_bytes!("../res/cursor.4bpp"));
    let bytes = aligned_bytes.as_u32_slice();
    let len = bytes.len() / 8;
    let tiles = unsafe { slice::from_raw_parts(bytes.as_ptr() as *const [u32; 8], len) };
    OBJ_TILES
        .as_region()
        .sub_slice(..len)
        .write_from_slice(tiles);

    // Draw the initial game state.
    state.draw();

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

    let mut prev_keys = KeyInput::new();

    loop {
        // Read keys for each frame.
        let keys = KEYINPUT.read();

        if keys.start() && !prev_keys.start() {
            log::info!("cursor: {:?}", state.cursor);
        }
        const MAX_POSITION: Position = Position { x: 15, y: 15 };
        if keys.right() && !prev_keys.right() {
            state.cursor = state.cursor.move_saturating(Direction::Right, MAX_POSITION);
        }
        if keys.up() && !prev_keys.up() {
            state.cursor = state.cursor.move_saturating(Direction::Up, MAX_POSITION);
        }
        if keys.left() && !prev_keys.left() {
            state.cursor = state.cursor.move_saturating(Direction::Left, MAX_POSITION);
        }
        if keys.down() && !prev_keys.down() {
            state.cursor = state.cursor.move_saturating(Direction::Down, MAX_POSITION);
        }
        if keys.a() && !prev_keys.a() {
            if state
                .game
                .execute_turn(Turn {
                    rotate: state.cursor,
                })
                .is_ok()
            {
                state.draw();
            }
        }

        prev_keys = keys;

        VBlankIntrWait();

        // Draw the cursor.
        let mut obj = ObjAttr::new();
        obj.set_x(state.cursor.x as u16 * 16);
        obj.set_y(state.cursor.y as u16 * 16);
        obj.set_tile_id(0);
        obj.set_palbank(0);
        obj.1 = obj.1.with_size(1);
        OBJ_ATTR_ALL.get(0).unwrap().write(obj);
    }
}
