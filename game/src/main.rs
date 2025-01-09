#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, test_runner(gba_test::runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_harness")]

mod align;
mod bios;
mod game;
mod mmio;
mod random;
#[cfg(not(test))]
mod runtime;
mod screen;

use log::error;
use mmio::{interrupts::Interrupts, vram::DisplayStatus, DISPSTAT, IE, IME};
use screen::Screen;

static mut VBLANKS_REMAINING: u16 = 0;

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

    // Audio test.
    unsafe {
        mmio::AUDIO_CONTROL.write_volatile(mmio::audio::Control::new().sound_a_right(true).sound_a_left(true).sound_a_fifo_reset(true));
        mmio::AUDIO_ENABLE.write_volatile(mmio::audio::Enable::new().master_enable(true));
        let audio_bytes = include_bytes_aligned!("../res/audio/camera_zoom_in.bin").0;
        let (sample_rate_bytes, samples) = audio_bytes.split_first_chunk::<4>().unwrap();
        let sample_rate = u32::from_le_bytes(*sample_rate_bytes);
        mmio::DMA1_SOURCE.write_volatile(samples.as_ptr());
        mmio::DMA1_DESTINATION.write_volatile(mmio::AUDIO_FIFO_A.cast());
        mmio::DMA1_CNT.write_volatile(mmio::dma::DmaControl::new().with_destination_address_control(mmio::dma::AddressControl::Fixed).with_repeat().with_transfer_32bit().with_timing(mmio::dma::Timing::Special).with_enabled());

        const CLOCK: u32 = 1 << 24;
        let ticks_per_sample = CLOCK / sample_rate;
        mmio::TIMER0_COUNT.write_volatile((65536 - ticks_per_sample) as u16);
        mmio::TIMER0_CONTROL.write_volatile(mmio::timer::Control::new().with_prescalar(mmio::timer::Prescalar::Freq1).with_enable(true));
    }
    loop {}

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
