#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, test_runner(gba_test::runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_harness")]

mod align;
mod bios;
mod game;
mod mmio;
#[cfg(not(test))]
mod runtime;
mod screen;

use log::error;
use mmio::{interrupts::Interrupts, vram::DisplayStatus, DISPSTAT, IE, IME};
use screen::Screen;

/// Initialize logging in an emulator, if possible.
///
/// Note that we don't actually care if either of these loggers fails to initialize. We just want
/// one of them initialized if at all possible to make debugging easier.
pub fn init_log() {
    if mgba_log::init().is_err() {
        let _ = nocash_gba_log::init();
    }
}

/// This panic handler is specifically for debug mode.
///
/// When panicking in debug builds, the panic info is logged as an error. Following this, a fatal
/// log will occur to halt emulation and display an error to the user. This is done to ensure the
/// entirety of the panic info is displayed to the user, as mGBA only allows for up to 256 bytes to
/// be logged at once.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("{}", info);
    mgba_log::fatal!("Halting due to panic. See logs for `PanicInfo`.");
    loop {}
}

/// Entry point for the game.
#[cfg(not(test))]
#[no_mangle]
pub fn main() -> ! {
    // Initialize the global logger.
    //
    // This logging only works in mGBA. It is only enabled in debug builds.
    #[cfg(debug_assertions)]
    init_log();

    unsafe {
        // Enable vblank interrupts.
        DISPSTAT.write_volatile(DisplayStatus::ENABLE_VBLANK_INTERRUPTS);
        IE.write_volatile(Interrupts::VBLANK);
        // Enable interrupts generally.
        IME.write_volatile(true);
    }

    let mut screen = Screen::default();

    loop {
        screen.run();
    }
}

#[cfg(test)]
#[no_mangle]
pub fn main() {
    init_log();
    test_harness()
}
