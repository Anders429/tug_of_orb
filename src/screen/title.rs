use super::{Game, Screen};
use crate::{
    bios::wait_for_vblank,
    game,
    game::{Color, Grid, Position},
    include_bytes_aligned,
    mmio::{
        keys::KeyInput,
        vram::{BackgroundControl, BlendControl, ColorEffect, DisplayControl, TextScreenEntry},
        BG1CNT, BG2CNT, BG_PALETTE, BLDCNT, BLDY, CHARBLOCK0, DISPCNT, KEYINPUT,
        TEXT_SCREENBLOCK16, TEXT_SCREENBLOCK8,
    },
};
use core::mem::transmute;
use deranged::{RangedU16, RangedU8};

pub struct Title {
    random_seed: u32,
}

impl Title {
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

            BG1CNT.write_volatile(
                BackgroundControl::new()
                    .with_screenblock(RangedU8::new_static::<8>())
                    .with_priority(RangedU8::new_static::<1>()),
            );
            BG2CNT.write_volatile(
                BackgroundControl::new()
                    .with_screenblock(RangedU8::new_static::<16>())
                    .with_priority(RangedU8::new_static::<0>()),
            );
            DISPCNT.write_volatile(DisplayControl::new().with_bg1(true).with_bg2(true));

            // Load palettes.
            BG_PALETTE.write_volatile(transmute(include_bytes_aligned!("../../res/title.pal").0));
            BG_PALETTE
                .add(1)
                .write_volatile(transmute(include_bytes_aligned!("../../res/red.pal").0));
            BG_PALETTE
                .add(2)
                .write_volatile(transmute(include_bytes_aligned!("../../res/press_a.pal").0));
        }

        // Load tiles.
        unsafe {
            CHARBLOCK0
                .cast::<[[u32; 8]; 75]>()
                .write_volatile(transmute(include_bytes_aligned!("../../res/title.4bpp").0));
            CHARBLOCK0.add(75).write_volatile(transmute(
                include_bytes_aligned!("../../res/background.4bpp").0,
            ));
            CHARBLOCK0
                .add(76)
                .write_volatile(transmute(include_bytes_aligned!("../../res/empty.4bpp").0));
            CHARBLOCK0
                .add(77)
                .cast::<[[u32; 8]; 23]>()
                .write_volatile(transmute(
                    include_bytes_aligned!("../../res/press_a.4bpp").0,
                ));
        }

        // Draw white background.
        let screenblock8_ptr = TEXT_SCREENBLOCK8.cast::<[TextScreenEntry; 30]>();
        let screenblock16_ptr = TEXT_SCREENBLOCK16.cast::<[TextScreenEntry; 30]>();
        for i in 0..20 {
            unsafe {
                screenblock8_ptr.byte_add(64 * i).write_volatile(
                    [TextScreenEntry::new()
                        .with_tile(RangedU16::new_static::<75>())
                        .with_palette(RangedU8::new_static::<1>()); 30],
                );
                screenblock16_ptr.byte_add(64 * i).write_volatile(
                    [TextScreenEntry::new()
                        .with_tile(RangedU16::new_static::<76>())
                        .with_palette(RangedU8::new_static::<0>()); 30],
                );
            }
        }

        // Draw the logo.
        for (index, row) in unsafe {
            transmute::<_, [[TextScreenEntry; 10]; 10]>(
                include_bytes_aligned!("../../res/title.map").0,
            )
        }
        .into_iter()
        .enumerate()
        {
            unsafe {
                TEXT_SCREENBLOCK16
                    .add((index + 2) * 32 + 10)
                    .cast::<[TextScreenEntry; 10]>()
                    .write_volatile(row);
            }
        }

        // Draw the press a.
        for (index, row) in unsafe {
            transmute::<_, [[TextScreenEntry; 12]; 2]>(
                include_bytes_aligned!("../../res/press_a.map").0,
            )
        }
        .into_iter()
        .enumerate()
        {
            unsafe {
                TEXT_SCREENBLOCK16
                    .add((index + 16) * 32 + 10)
                    .cast::<[TextScreenEntry; 12]>()
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

        Self { random_seed: 0 }
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

            return Some(Screen::Game(Game::new(
                Position { x: 0, y: 0 },
                game::Game::builder()
                    .grid(Grid::generate(self.random_seed))
                    .build(),
                Color::Red,
            )));
        }

        self.random_seed += 1;

        None
    }
}
