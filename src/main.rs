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
use log::error;
use screen::Screen;

/// This panic handler is specifically for debug mode.
///
/// When panicking in debug builds, the panic info is logged as an error. Following this, a fatal
/// log will occur to halt emulation and display an error to the user. This is done to ensure the
/// entirety of the panic info is displayed to the user, as mGBA only allows for up to 256 bytes to
/// be logged at once.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("{}", info);
    mgba_log::fatal!("Halting due to panic. See logs for `PanicInfo`.");
    loop {}
}

/// Entry point for the game.
#[no_mangle]
pub fn main() -> ! {
    // Initialize the global logger.
    //
    // This logging only works in mGBA. It is only enabled in debug builds.
    #[cfg(debug_assertions)]
    let _ = mgba_log::init();

    // Enable vblank interrupts.
    DISPSTAT.write(DisplayStatus::new().with_irq_vblank(true));
    IE.write(IrqBits::VBLANK);
    // Enable interrupts generally.
    IME.write(true);

    let mut screen = Screen::default();

    loop {
        screen.run();
    }
}

#[no_mangle]
pub fn __sync_synchronize() {}
