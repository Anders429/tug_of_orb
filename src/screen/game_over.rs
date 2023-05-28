use super::{Screen, Title};
use crate::{
    game,
    game::{Color, Grid, Position},
};
use core::slice;
use gba::{
    bios::VBlankIntrWait,
    mmio::{
        bg_palbank, obj_palbank, BG0CNT, BG1CNT, BG1HOFS, BG1VOFS, BG2CNT, BG2HOFS, BG2VOFS,
        BG3CNT, BLDCNT, BLDY, CHARBLOCK0_4BPP, DISPCNT, KEYINPUT, OBJ_ATTR0, OBJ_ATTR_ALL,
        OBJ_TILES, TEXT_SCREENBLOCKS,
    },
    video::{
        BackgroundControl, BlendControl, ColorEffectMode, DisplayControl, TextEntry, VideoMode,
    },
    Align4,
};

pub enum PlayerResult {
    Win,
    Lose,
}

pub struct GameOver;

impl GameOver {
    pub fn new(result: PlayerResult) -> Self {
        // Set up background layers.
        BG0CNT.write(
            BackgroundControl::new()
                .with_screenblock(8)
                .with_priority(3),
        );
        BG1CNT.write(
            BackgroundControl::new()
                .with_screenblock(16)
                .with_priority(2)
                .with_size(3),
        );
        BG2CNT.write(
            BackgroundControl::new()
                .with_screenblock(24)
                .with_priority(1)
                .with_size(3),
        );
        BG3CNT.write(
            BackgroundControl::new()
                .with_screenblock(28)
                .with_priority(0),
        );
        DISPCNT.write(
            DisplayControl::new()
                .with_show_bg0(true)
                .with_show_bg1(true)
                .with_show_bg2(true)
                .with_show_bg3(true),
        );

        // Load palettes.
        for (index, bytes) in Align4(*include_bytes!("../../res/win.pal"))
            .as_u16_slice()
            .iter()
            .enumerate()
        {
            bg_palbank(5)
                .get(index)
                .unwrap()
                .write(gba::video::Color(*bytes));
        }
        for (index, bytes) in Align4(*include_bytes!("../../res/lose.pal"))
            .as_u16_slice()
            .iter()
            .enumerate()
        {
            bg_palbank(6)
                .get(index)
                .unwrap()
                .write(gba::video::Color(*bytes));
        }

        // Load win.
        let aligned_bytes = Align4(*include_bytes!("../../res/win.4bpp"));
        let bytes = aligned_bytes.as_u32_slice();
        let len = bytes.len() / 8;
        let tiles = unsafe { slice::from_raw_parts(bytes.as_ptr() as *const [u32; 8], len) };
        CHARBLOCK0_4BPP
            .as_region()
            .sub_slice(58..58 + len)
            .write_from_slice(&tiles[0..len]);

        // Load lose.
        let aligned_bytes = Align4(*include_bytes!("../../res/lose.4bpp"));
        let bytes = aligned_bytes.as_u32_slice();
        let len = bytes.len() / 8;
        let tiles = unsafe { slice::from_raw_parts(bytes.as_ptr() as *const [u32; 8], len) };
        CHARBLOCK0_4BPP
            .as_region()
            .sub_slice(72..72 + len)
            .write_from_slice(&tiles[0..len]);

        // Display.
        match result {
            PlayerResult::Win => {
                for (index, tile) in Align4(*include_bytes!("../../res/win.map"))
                    .as_u16_slice()
                    .iter()
                    .enumerate()
                {
                    let x = index % 8;
                    let y = index / 8;
                    TEXT_SCREENBLOCKS
                        .get_frame(28)
                        .unwrap()
                        .get_row(y + 9)
                        .unwrap()
                        .get(x + 11)
                        .unwrap()
                        .write(TextEntry::new().with_tile(*tile).with_palbank(5));
                }
            }
            PlayerResult::Lose => {
                for (index, tile) in Align4(*include_bytes!("../../res/lose.map"))
                    .as_u16_slice()
                    .iter()
                    .enumerate()
                {
                    let x = index % 8;
                    let y = index / 8;
                    TEXT_SCREENBLOCKS
                        .get_frame(28)
                        .unwrap()
                        .get_row(y + 9)
                        .unwrap()
                        .get(x + 11)
                        .unwrap()
                        .write(TextEntry::new().with_tile(*tile).with_palbank(6));
                }
            }
        }

        Self
    }

    pub fn run(&mut self) -> Option<Screen> {
        let keys = KEYINPUT.read();
        if keys.a() {
            // Fade out.
            VBlankIntrWait();
            for fade in (0..31) {
                VBlankIntrWait();
                BLDY.write(fade / 2);
            }

            // Reset scroll.
            BG1HOFS.write(0);
            BG1VOFS.write(0);
            BG2HOFS.write(0);
            BG2VOFS.write(0);

            return Some(Screen::Title(Title::new()));
        }

        None
    }
}
