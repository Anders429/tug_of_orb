#![no_std]
#![no_main]

#[cfg(debug_assertions)]
use ::{
    core::fmt::Write,
    log::error,
    mgba_log::{
        MgbaLogLevel,
        MgbaWriter,
    }
};
use gba::{bios::VBlankIntrWait, interrupts::IrqBits, mmio::{
    BG0CNT,
    DISPCNT,
    DISPSTAT,
    IE,
    IME,
    KEYINPUT,
}, video::{BackgroundControl, DisplayControl, DisplayStatus, VideoMode}};

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
enum Alignment {
    None,
    Red,
    Blue,
    Yellow,
    Green,
}

impl Default for Alignment {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Default)]
struct Square {
    alignment: Alignment,
}

#[derive(Debug)]
struct Board {
    grid: [[Square; 16]; 16],
}

impl Board {
    fn new() -> Self {
        Self {
            grid: Default::default(),
        }
    }
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
    DISPCNT.write(DisplayControl::new().with_video_mode(VideoMode::_2).with_show_bg0(true));

    let board = Board::new();

    loop {
        // Read keys for each frame.
        let keys = KEYINPUT.read();
        
        VBlankIntrWait();

        // Draw the grid.
        // for row in &board.grid {
        //     for square in row {
                
        //     }
        // }
    }
}
