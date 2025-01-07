use super::{Screen, Title};
use crate::{
    bios::wait_for_vblank,
    include_bytes_aligned,
    mmio::{
        keys::KeyInput,
        vram::{BackgroundControl, DisplayControl, TextScreenEntry},
        BG0CNT, BG1CNT, BG1HOFS, BG1VOFS, BG2CNT, BG2HOFS, BG2VOFS, BG3CNT, BG_PALETTE, BLDY,
        CHARBLOCK0, DISPCNT, KEYINPUT, TEXT_SCREENBLOCK28,
    },
};
use core::mem::transmute;
use deranged::{RangedU16, RangedU8};

pub enum PlayerResult {
    Win,
    Lose,
}

pub struct GameOver;

impl GameOver {
    pub fn new(result: PlayerResult) -> Self {
        unsafe {
            // Set up background layers.
            BG0CNT.write_volatile(
                BackgroundControl::new()
                    .with_screenblock(RangedU8::new_static::<8>())
                    .with_priority(RangedU8::new_static::<3>()),
            );
            BG1CNT.write_volatile(
                BackgroundControl::new()
                    .with_screenblock(RangedU8::new_static::<16>())
                    .with_priority(RangedU8::new_static::<2>())
                    .with_screen_size(RangedU8::new_static::<3>()),
            );
            BG2CNT.write_volatile(
                BackgroundControl::new()
                    .with_screenblock(RangedU8::new_static::<24>())
                    .with_priority(RangedU8::new_static::<1>())
                    .with_screen_size(RangedU8::new_static::<3>()),
            );
            BG3CNT.write_volatile(
                BackgroundControl::new()
                    .with_screenblock(RangedU8::new_static::<28>())
                    .with_priority(RangedU8::new_static::<0>()),
            );
            DISPCNT.write_volatile(
                DisplayControl::new()
                    .with_bg0(true)
                    .with_bg1(true)
                    .with_bg2(true)
                    .with_bg3(true),
            );

            // Load palettes.
            BG_PALETTE
                .add(5)
                .write_volatile(transmute(include_bytes_aligned!("../../res/win.pal").0));
            BG_PALETTE
                .add(6)
                .write_volatile(transmute(include_bytes_aligned!("../../res/lose.pal").0));
        }

        unsafe {
            // Load win.
            CHARBLOCK0
                .add(58)
                .cast::<[[u32; 8]; 14]>()
                .write_volatile(transmute(include_bytes_aligned!("../../res/win.4bpp").0));
            // Load lose.
            CHARBLOCK0
                .add(72)
                .cast::<[[u32; 8]; 16]>()
                .write_volatile(transmute(include_bytes_aligned!("../../res/lose.4bpp").0));
        }

        // Display.
        match result {
            PlayerResult::Win => {
                for (index, row) in unsafe {
                    transmute::<_, [[TextScreenEntry; 8]; 2]>(
                        include_bytes_aligned!("../../res/win.map").0,
                    )
                }
                .into_iter()
                .enumerate()
                {
                    unsafe {
                        TEXT_SCREENBLOCK28
                            .add((index + 9) * 32 + 10)
                            .cast::<[TextScreenEntry; 8]>()
                            .write_volatile(row);
                    }
                }
            }
            PlayerResult::Lose => {
                for (index, row) in unsafe {
                    transmute::<_, [[TextScreenEntry; 8]; 2]>(
                        include_bytes_aligned!("../../res/lose.map").0,
                    )
                }
                .into_iter()
                .enumerate()
                {
                    unsafe {
                        TEXT_SCREENBLOCK28
                            .add((index + 9) * 32 + 10)
                            .cast::<[TextScreenEntry; 8]>()
                            .write_volatile(row);
                    }
                }
            }
        }

        Self
    }

    pub fn run(&mut self) -> Option<Screen> {
        let keys = unsafe { KEYINPUT.read_volatile() };
        if keys.contains(KeyInput::A) {
            // Fade out.
            wait_for_vblank();
            for fade in 0..31 {
                wait_for_vblank();
                unsafe {
                    BLDY.write_volatile(RangedU8::new_unchecked(fade / 2));
                }
            }

            // Reset scroll.
            unsafe {
                BG1HOFS.write_volatile(RangedU16::new_static::<0>());
                BG1VOFS.write_volatile(RangedU16::new_static::<0>());
                BG2HOFS.write_volatile(RangedU16::new_static::<0>());
                BG2VOFS.write_volatile(RangedU16::new_static::<0>());
            }

            return Some(Screen::Title(Title::new()));
        }

        None
    }
}
