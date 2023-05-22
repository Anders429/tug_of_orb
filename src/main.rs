#![no_std]
#![no_main]

mod game;

use game::{Direction, Game, Position};
use gba::{
    bios::VBlankIntrWait,
    interrupts::IrqBits,
    mmio::{BG0CNT, DISPCNT, DISPSTAT, IE, IME, KEYINPUT},
    video::{BackgroundControl, DisplayControl, DisplayStatus, VideoMode},
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
            .with_video_mode(VideoMode::_2)
            .with_show_bg0(true),
    );

    let mut state = State {
        game: Game::builder().build(),

        cursor: Position { x: 0, y: 0 },
    };
    log::info!("{:?}", state);

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

        // Draw the grid.
        // for row in &board.grid {
        //     for square in row {

        //     }
        // }
    }
}
