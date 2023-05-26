#![no_std]
#![no_main]

mod game;
mod screen;

use game::{Color, Direction, Game, Grid, Node, Position};
use gba::{
    interrupts::IrqBits,
    mmio::{DISPSTAT, IE, IME},
    video::DisplayStatus,
};
use screen::Screen;
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

    let mut screen = Screen::Game(screen::Game::new(
        Position { x: 0, y: 0 },
        Game::builder()
            .grid(Grid::new({
                let mut grid = [[Node::Arrow {
                    direction: Direction::Right,
                    alignment: None,
                }; 16]; 16];
                grid[0][0] = Node::Arrow {
                    direction: Direction::Up,
                    alignment: Some(Color::Red),
                };
                grid[0][1] = Node::Arrow {
                    direction: Direction::Up,
                    alignment: Some(Color::Blue),
                };
                grid[0][2] = Node::Arrow {
                    direction: Direction::Up,
                    alignment: Some(Color::Yellow),
                };
                grid[0][3] = Node::Arrow {
                    direction: Direction::Up,
                    alignment: Some(Color::Green),
                };
                grid
            }))
            .build(),
    ));

    loop {
        screen.run();
    }
}
