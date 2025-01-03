//! Memory-mapped I/O addresses and types.

pub mod dma;
pub mod interrupts;
pub mod keys;
pub mod vram;

use deranged::{RangedU16, RangedU8};
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
