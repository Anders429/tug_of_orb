//! Functions and types to interract with the GBA BIOS.

use core::arch::asm;

/// Waits until a new v-blank interrupt occurs.
#[instruction_set(arm::t32)]
pub fn wait_for_vblank() {
    unsafe {
        asm! {
            "swi #0x05",
            out("r0") _,
            out("r1") _,
            out("r3") _,
            options(preserves_flags),
        }
    };
}
