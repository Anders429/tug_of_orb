#![no_std]
#![no_main]

mod game;

use game::{Direction, Game, Position};
use gba::{
    bios::VBlankIntrWait,
    interrupts::IrqBits,
    mmio::{
        obj_palbank, BG0CNT, DISPCNT, DISPSTAT, IE, IME, KEYINPUT, OBJ_ATTR0, OBJ_ATTR_ALL,
        OBJ_TILES,
    },
    video::{
        obj::{ObjAttr, ObjAttr0, ObjDisplayStyle},
        BackgroundControl, Color, DisplayControl, DisplayStatus, VideoMode,
    },
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
    BG0CNT.write(BackgroundControl::new().with_screenblock(31));
    // Set BG0 to be displayed.
    DISPCNT.write(
        DisplayControl::new()
            .with_show_bg0(true)
            .with_show_obj(true)
            .with_obj_vram_1d(true),
    );

    let mut state = State {
        game: Game::builder().build(),

        cursor: Position { x: 0, y: 0 },
    };
    log::info!("{:?}", state);

    // Define a palette.
    let colors = [Color::from_rgb(31, 16, 1), Color::from_rgb(3, 3, 3)];
    for (index, color) in colors.iter().enumerate() {
        obj_palbank(0).index(index + 1).write(*color);
    }

    // Define the tiles.
    //
    // There has got to be some better way. Need to figure out how to convert resources to 4bpp.
    OBJ_TILES.as_region().get(0).unwrap().write([
        0b0000_0000_0010_0010_0010_0010_0010_0010,
        0b0000_0000_0010_0001_0001_0001_0001_0010,
        0b0000_0000_0010_0010_0010_0001_0001_0010,
        0b0000_0000_0000_0000_0010_0010_0001_0010,
        0b0000_0000_0000_0000_0000_0010_0001_0010,
        0b0000_0000_0000_0000_0000_0010_0010_0010,
        0b0000_0000_0000_0000_0000_0000_0000_0000,
        0b0000_0000_0000_0000_0000_0000_0000_0000,
    ]);
    OBJ_TILES.as_region().get(1).unwrap().write([
        0b0010_0010_0010_0010_0010_0010_0000_0000,
        0b0010_0001_0001_0001_0001_0010_0000_0000,
        0b0010_0001_0001_0010_0010_0010_0000_0000,
        0b0010_0001_0010_0010_0000_0000_0000_0000,
        0b0010_0001_0010_0000_0000_0000_0000_0000,
        0b0010_0010_0010_0000_0000_0000_0000_0000,
        0b0000_0000_0000_0000_0000_0000_0000_0000,
        0b0000_0000_0000_0000_0000_0000_0000_0000,
    ]);
    OBJ_TILES.as_region().get(2).unwrap().write([
        0b0000_0000_0000_0000_0000_0000_0000_0000,
        0b0000_0000_0000_0000_0000_0000_0000_0000,
        0b0000_0000_0000_0000_0000_0010_0010_0010,
        0b0000_0000_0000_0000_0000_0010_0001_0010,
        0b0000_0000_0000_0000_0010_0010_0001_0010,
        0b0000_0000_0010_0010_0010_0001_0001_0010,
        0b0000_0000_0010_0001_0001_0001_0001_0010,
        0b0000_0000_0010_0010_0010_0010_0010_0010,
    ]);
    OBJ_TILES.as_region().get(3).unwrap().write([
        0b0000_0000_0000_0000_0000_0000_0000_0000,
        0b0000_0000_0000_0000_0000_0000_0000_0000,
        0b0010_0010_0010_0000_0000_0000_0000_0000,
        0b0010_0001_0010_0000_0000_0000_0000_0000,
        0b0010_0001_0010_0010_0000_0000_0000_0000,
        0b0010_0001_0001_0010_0010_0010_0000_0000,
        0b0010_0001_0001_0001_0001_0010_0000_0000,
        0b0010_0010_0010_0010_0010_0010_0000_0000,
    ]);

    // Define the object.
    let mut obj = ObjAttr::new();
    obj.set_x(44);
    obj.set_y(38);
    obj.set_tile_id(0);
    obj.set_palbank(0);
    obj.1 = obj.1.with_size(1);
    OBJ_ATTR_ALL.get(0).unwrap().write(obj);

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
    }
}
