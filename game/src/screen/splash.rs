use super::{Screen, Title};
use crate::{
    bios::wait_for_vblank,
    include_bytes_aligned,
    mmio::{
        keys::KeyInput,
        vram::{
            BackgroundControl, BlendControl, Color, ColorEffect, DisplayControl, TextScreenEntry,
        },
        BG2CNT, BG_PALETTE, BLDCNT, BLDY, CHARBLOCK0, DISPCNT, KEYINPUT, TEXT_SCREENBLOCK8,
    },
};
use core::mem::transmute;
use deranged::RangedU8;

pub struct Splash {
    frame_count: u16,
}

impl Splash {
    pub fn new() -> Self {
        unsafe {
            // Initialize fade.
            BLDCNT.write_volatile(
                BlendControl::new()
                    .with_target1_bg0(true)
                    .with_target1_bg1(true)
                    .with_target1_bg2(true)
                    .with_target1_bg3(true)
                    .with_target1_obj(true)
                    .with_target1_backdrop(true)
                    .with_color_effect(ColorEffect::Brighten),
            );
            // Fade out while we set up the screen.
            BLDY.write_volatile(RangedU8::new_static::<16>());

            BG2CNT.write_volatile(
                BackgroundControl::new()
                    .with_screenblock(RangedU8::new_static::<8>())
                    .with_8bpp(true),
            );
            DISPCNT.write_volatile(DisplayControl::new().with_bg2(true));

            // Load palette.
            BG_PALETTE.cast::<[Color; 256]>().write_volatile(transmute(
                include_bytes_aligned!("../../res/splash_jam.pal").0,
            ));
        }

        // Load tiles.
        unsafe {
            CHARBLOCK0
                .cast::<[[u32; 16]; 161]>()
                .write_volatile(transmute(
                    include_bytes_aligned!("../../res/splash_jam.8bpp").0,
                ));
        }

        // Draw the logo.
        for (index, row) in unsafe {
            transmute::<_, [[TextScreenEntry; 30]; 20]>(
                include_bytes_aligned!("../../res/splash_jam.map").0,
            )
        }
        .into_iter()
        .enumerate()
        {
            unsafe {
                TEXT_SCREENBLOCK8
                    .add(32 * index)
                    .cast::<[TextScreenEntry; 30]>()
                    .write_volatile(row);
            }
        }

        // Fade in.
        for fade in (0..31).rev() {
            wait_for_vblank();
            unsafe {
                BLDY.write_volatile(RangedU8::new_unchecked(fade / 2));
            }
        }

        Self { frame_count: 0 }
    }

    pub fn run(&mut self) -> Option<Screen> {
        let keys = unsafe { KEYINPUT.read_volatile() };
        if self.frame_count > 180 || keys.contains(KeyInput::A) {
            wait_for_vblank();

            // Fade out.
            wait_for_vblank();
            for fade in 0..31 {
                wait_for_vblank();
                unsafe {
                    BLDY.write_volatile(RangedU8::new_unchecked(fade / 2));
                }
            }

            Some(Screen::Title(Title::new()))
        } else {
            wait_for_vblank();
            self.frame_count += 1;
            None
        }
    }
}
