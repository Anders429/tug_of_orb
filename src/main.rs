#![no_std]
#![no_main]

mod game;

use core::slice;
use game::{Direction, Game, Grid, Node, Position};
use gba::{
    bios::VBlankIntrWait,
    interrupts::IrqBits,
    mmio::{
        bg_palbank, obj_palbank, BG0CNT, CHARBLOCK0_4BPP, DISPCNT, DISPSTAT, IE, IME, KEYINPUT,
        OBJ_ATTR0, OBJ_ATTR_ALL, OBJ_TILES, TEXT_SCREENBLOCKS,
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

#[derive(Debug)]
struct State {
    cursor: game::Position,

    game: Game,
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

    // Configure BG0 tilemap.
    BG0CNT.write(BackgroundControl::new().with_screenblock(8));
    // Set BG0 to be displayed.
    DISPCNT.write(
        DisplayControl::new()
            .with_show_bg0(true)
            .with_show_obj(true)
            .with_obj_vram_1d(true),
    );

    let mut state = State {
        game: Game::builder()
            .grid(Grid::new([[Node::Wall; 16]; 16]))
            .build(),

        cursor: Position { x: 0, y: 0 },
    };
    log::info!("{:?}", state);

    // Define the neutral palette.
    for (index, bytes) in Align4(*include_bytes!("../res/neutral.pal"))
        .as_u16_slice()
        .iter()
        .enumerate()
    {
        bg_palbank(0).index(index).write(Color(*bytes));
    }

    // Define cursor palette.
    for (index, bytes) in Align4(*include_bytes!("../res/cursor.pal"))
        .as_u16_slice()
        .iter()
        .enumerate()
    {
        obj_palbank(0).index(index).write(Color(*bytes));
    }

    // Define the wall tiles.
    let aligned_bytes = Align4(*include_bytes!("../res/wall.4bpp"));
    let bytes = aligned_bytes.as_u32_slice();
    let len = bytes.len() / 8;
    let tiles = unsafe { slice::from_raw_parts(bytes.as_ptr() as *const [u32; 8], len) };
    CHARBLOCK0_4BPP
        .as_region()
        .sub_slice(1..len + 1)
        .write_from_slice(tiles);

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
    for (y, row) in state.game.grid().iter().enumerate() {
        for (x, node) in row.iter().enumerate() {
            match node {
                Node::Wall => {
                    TEXT_SCREENBLOCKS
                        .get_frame(8)
                        .unwrap()
                        .get_row(y * 2)
                        .unwrap()
                        .get(x * 2)
                        .unwrap()
                        .write(TextEntry::new().with_tile(1));
                    TEXT_SCREENBLOCKS
                        .get_frame(8)
                        .unwrap()
                        .get_row(y * 2)
                        .unwrap()
                        .get(x * 2 + 1)
                        .unwrap()
                        .write(TextEntry::new().with_tile(2));
                    TEXT_SCREENBLOCKS
                        .get_frame(8)
                        .unwrap()
                        .get_row(y * 2 + 1)
                        .unwrap()
                        .get(x * 2)
                        .unwrap()
                        .write(TextEntry::new().with_tile(3));
                    TEXT_SCREENBLOCKS
                        .get_frame(8)
                        .unwrap()
                        .get_row(y * 2 + 1)
                        .unwrap()
                        .get(x * 2 + 1)
                        .unwrap()
                        .write(TextEntry::new().with_tile(4));
                }
                _ => {}
            }
        }
    }

    // Hide other objects.
    OBJ_ATTR0.iter().skip(1).for_each(|address| {
        address.write(ObjAttr0::new().with_style(ObjDisplayStyle::NotDisplayed))
    });

    loop {
        // Read keys for each frame.
        let keys = KEYINPUT.read();

        if keys.start() {
            log::info!("cursor: {:?}", state.cursor);
        }
        const MAX_POSITION: Position = Position { x: 15, y: 15 };
        if keys.right() {
            state.cursor = state.cursor.move_saturating(Direction::Right, MAX_POSITION);
        }
        if keys.up() {
            state.cursor = state.cursor.move_saturating(Direction::Up, MAX_POSITION);
        }
        if keys.left() {
            state.cursor = state.cursor.move_saturating(Direction::Left, MAX_POSITION);
        }
        if keys.down() {
            state.cursor = state.cursor.move_saturating(Direction::Down, MAX_POSITION);
        }

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
