use super::{Game, Screen};
use crate::{
    game,
    game::{Color, Grid, Position},
};
use core::slice;
use gba::{
    bios::VBlankIntrWait,
    mmio::{
        bg_palbank, BG1CNT, BG2CNT, BG_PALETTE, BLDCNT, BLDY, CHARBLOCK0_4BPP, CHARBLOCK1_4BPP,
        CHARBLOCK2_8BPP, DISPCNT, KEYINPUT, TEXT_SCREENBLOCKS,
    },
    video::{
        BackgroundControl, BlendControl, ColorEffectMode, DisplayControl, TextEntry, VideoMode,
    },
    Align4,
};

pub struct Title {
    random_seed: u32,
}

impl Title {
    pub fn new() -> Self {
        // Initialize fade.
        BLDCNT.write(
            BlendControl::new()
                .with_target1_bg0(true)
                .with_target1_bg1(true)
                .with_target1_bg2(true)
                .with_target1_bg3(true)
                .with_target1_obj(true)
                .with_target1_backdrop(true)
                .with_mode(ColorEffectMode::Brighten),
        );
        // Fade out while we set up the screen.
        BLDY.write(16);

        BG1CNT.write(
            BackgroundControl::new()
                .with_screenblock(8)
                .with_priority(1),
        );
        BG2CNT.write(
            BackgroundControl::new()
                .with_screenblock(16)
                .with_priority(0),
        );
        DISPCNT.write(
            DisplayControl::new()
                .with_show_bg1(true)
                .with_show_bg2(true),
        );

        // Load palette.
        for (index, bytes) in Align4(*include_bytes!("../../res/title.pal"))
            .as_u16_slice()
            .iter()
            .enumerate()
        {
            bg_palbank(0)
                .get(index)
                .unwrap()
                .write(gba::video::Color(*bytes));
        }
        // Load red palette.
        for (index, bytes) in Align4(*include_bytes!("../../res/red.pal"))
            .as_u16_slice()
            .iter()
            .enumerate()
        {
            bg_palbank(1).index(index).write(gba::video::Color(*bytes));
        }
        // Load press a palette.
        for (index, bytes) in Align4(*include_bytes!("../../res/press_a.pal"))
            .as_u16_slice()
            .iter()
            .enumerate()
        {
            bg_palbank(2).index(index).write(gba::video::Color(*bytes));
        }

        // Load tiles.
        let aligned_bytes = Align4(*include_bytes!("../../res/title.4bpp"));
        let bytes = aligned_bytes.as_u32_slice();
        let len = bytes.len() / 8;
        let tiles = unsafe { slice::from_raw_parts(bytes.as_ptr() as *const [u32; 8], len) };
        CHARBLOCK0_4BPP
            .as_region()
            .sub_slice(0..len)
            .write_from_slice(&tiles[0..len]);

        let aligned_bytes = Align4(*include_bytes!("../../res/background.4bpp"));
        let bytes = aligned_bytes.as_u32_slice();
        let len = bytes.len() / 8;
        let tiles = unsafe { slice::from_raw_parts(bytes.as_ptr() as *const [u32; 8], len) };
        CHARBLOCK0_4BPP
            .as_region()
            .sub_slice(75..75 + len)
            .write_from_slice(&tiles[0..len]);

        let aligned_bytes = Align4(*include_bytes!("../../res/empty.4bpp"));
        let bytes = aligned_bytes.as_u32_slice();
        let len = bytes.len() / 8;
        let tiles = unsafe { slice::from_raw_parts(bytes.as_ptr() as *const [u32; 8], len) };
        CHARBLOCK0_4BPP
            .as_region()
            .sub_slice(76..76 + len)
            .write_from_slice(&tiles[0..len]);

        let aligned_bytes = Align4(*include_bytes!("../../res/press_a.4bpp"));
        let bytes = aligned_bytes.as_u32_slice();
        let len = bytes.len() / 8;
        let tiles = unsafe { slice::from_raw_parts(bytes.as_ptr() as *const [u32; 8], len) };
        CHARBLOCK0_4BPP
            .as_region()
            .sub_slice(77..77 + len)
            .write_from_slice(&tiles[0..len]);

        // Draw white background.
        for y in 0..20 {
            for x in 0..30 {
                TEXT_SCREENBLOCKS
                    .get_frame(8)
                    .unwrap()
                    .get_row(y)
                    .unwrap()
                    .get(x)
                    .unwrap()
                    .write(TextEntry::new().with_tile(75).with_palbank(1));
                TEXT_SCREENBLOCKS
                    .get_frame(16)
                    .unwrap()
                    .get_row(y)
                    .unwrap()
                    .get(x)
                    .unwrap()
                    .write(TextEntry::new().with_tile(76).with_palbank(0));
            }
        }

        // Draw the logo.
        for (index, tile) in Align4(*include_bytes!("../../res/title.map"))
            .as_u16_slice()
            .iter()
            .enumerate()
        {
            let x = index % 10;
            let y = index / 10;
            TEXT_SCREENBLOCKS
                .get_frame(16)
                .unwrap()
                .get_row(y + 2)
                .unwrap()
                .get(x + 10)
                .unwrap()
                .write(TextEntry::new().with_tile(*tile));
        }

        // Draw the press a.
        for (index, tile) in Align4(*include_bytes!("../../res/press_a.map"))
            .as_u16_slice()
            .iter()
            .enumerate()
        {
            let x = index % 12;
            let y = index / 12;
            log::info!("{}, {}", x, y);
            TEXT_SCREENBLOCKS
                .get_frame(16)
                .unwrap()
                .get_row(y + 16)
                .unwrap()
                .get(x + 9)
                .unwrap()
                .write(TextEntry::new().with_tile(*tile).with_palbank(2));
        }

        // Fade in.
        for fade in (0..31).rev() {
            VBlankIntrWait();
            BLDY.write(fade / 2);
        }

        Self { random_seed: 0 }
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
