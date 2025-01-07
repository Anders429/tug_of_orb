//! Memory-mapped I/O addresses and types.

pub mod audio;
pub mod dma;
pub mod interrupts;
pub mod keys;
pub mod timer;
pub mod vram;

use deranged::{RangedU16, RangedU8};
use dma::DmaControl;
use interrupts::Interrupts;
use keys::KeyInput;
use vram::{
    BackgroundControl, BlendControl, Color, DisplayControl, DisplayStatus, ObjectAttributes,
    TextScreenEntry,
};

pub const DISPCNT: *mut DisplayControl = 0x0400_0000 as *mut DisplayControl;
pub const DISPSTAT: *mut DisplayStatus = 0x0400_0004 as *mut DisplayStatus;
pub const BG0CNT: *mut BackgroundControl = 0x0400_0008 as *mut BackgroundControl;
pub const BG1CNT: *mut BackgroundControl = 0x0400_000A as *mut BackgroundControl;
pub const BG2CNT: *mut BackgroundControl = 0x0400_000C as *mut BackgroundControl;
pub const BG3CNT: *mut BackgroundControl = 0x0400_000E as *mut BackgroundControl;
pub const BG1HOFS: *mut RangedU16<0, 511> = 0x0400_0014 as *mut RangedU16<0, 511>;
pub const BG1VOFS: *mut RangedU16<0, 511> = 0x0400_0016 as *mut RangedU16<0, 511>;
pub const BG2HOFS: *mut RangedU16<0, 511> = 0x0400_0018 as *mut RangedU16<0, 511>;
pub const BG2VOFS: *mut RangedU16<0, 511> = 0x0400_001A as *mut RangedU16<0, 511>;
pub const BLDCNT: *mut BlendControl = 0x0400_0050 as *mut BlendControl;
pub const BLDY: *mut RangedU8<0, 16> = 0x0400_0054 as *mut RangedU8<0, 16>;
pub const AUDIO_CONTROL: *mut audio::Control = 0x0400_0082 as *mut audio::Control;
pub const AUDIO_ENABLE: *mut audio::Enable = 0x0400_0084 as *mut audio::Enable;
pub const AUDIO_FIFO_A: *mut u32 = 0x0400_00A0 as *mut u32;
pub const DMA1_SOURCE: *mut *const u8 = 0x0400_00BC as *mut *const u8;
pub const DMA1_DESTINATION: *mut *mut u8 = 0x0400_00C0 as *mut *mut u8;
pub const DMA1_CNT: *mut DmaControl = 0x0400_00C6 as *mut DmaControl;
pub const TIMER0_COUNT: *mut u16 = 0x0400_0100 as *mut u16;
pub const TIMER0_CONTROL: *mut timer::Control = 0x0400_0102 as *mut timer::Control;
pub const KEYINPUT: *mut KeyInput = 0x0400_0130 as *mut KeyInput;
pub const IE: *mut Interrupts = 0x0400_0200 as *mut Interrupts;
pub const IME: *mut bool = 0x0400_0208 as *mut bool;
pub const BG_PALETTE: *mut [Color; 16] = 0x0500_0000 as *mut [Color; 16];
pub const OBJ_PALETTE: *mut [Color; 16] = 0x0500_0200 as *mut [Color; 16];
pub const CHARBLOCK0: *mut [u32; 8] = 0x0600_0000 as *mut [u32; 8];
pub const TEXT_SCREENBLOCK0: *mut TextScreenEntry = 0x0600_0000 as *mut TextScreenEntry;
pub const TEXT_SCREENBLOCK8: *mut TextScreenEntry = 0x0600_4000 as *mut TextScreenEntry;
pub const TEXT_SCREENBLOCK16: *mut TextScreenEntry = 0x0600_8000 as *mut TextScreenEntry;
pub const TEXT_SCREENBLOCK24: *mut TextScreenEntry = 0x0600_C000 as *mut TextScreenEntry;
pub const TEXT_SCREENBLOCK28: *mut TextScreenEntry = 0x0600_E000 as *mut TextScreenEntry;
pub const OBJ_TILES: *mut [u32; 8] = 0x0601_0000 as *mut [u32; 8];
pub const OBJ_ATTRS: *mut ObjectAttributes = 0x0700_0000 as *mut ObjectAttributes;
