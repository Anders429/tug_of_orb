use super::{Game, Screen};
use core::slice;
use crate::{
    game,
    game::{Color, Grid, Position},
};
use gba::{
    bios::VBlankIntrWait,
    mmio::{bg_palbank, BG2CNT, BLDCNT, BLDY, CHARBLOCK0_8BPP, CHARBLOCK1_8BPP, CHARBLOCK2_8BPP, DISPCNT, KEYINPUT, BG_PALETTE, TEXT_SCREENBLOCKS},
    video::{BackgroundControl, BlendControl, ColorEffectMode, DisplayControl, TextEntry, VideoMode},
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

        BG2CNT.write(
            BackgroundControl::new()
                .with_screenblock(8)
                .with_bpp8(true)
        );
        DISPCNT.write(
            DisplayControl::new()
                .with_show_bg2(true)
        );

        // Load palette.
        for (index, bytes) in Align4(*include_bytes!("../../res/splash_jam.pal"))
            .as_u16_slice()
            .iter()
            .enumerate()
        {
            BG_PALETTE.get(index).unwrap().write(gba::video::Color(*bytes));
        }

        // Load tiles.
        let aligned_bytes = Align4(*include_bytes!("../../res/splash_jam.8bpp"));
        let bytes = aligned_bytes.as_u32_slice();
        let len = bytes.len() / 16;
        let tiles = unsafe { slice::from_raw_parts(bytes.as_ptr() as *const [u32; 16], len) };
        CHARBLOCK0_8BPP
            .as_region()
            .sub_slice(0..len)
            .write_from_slice(&tiles[0..len]);

        // Draw the logo.
        for (index, tile) in Align4(*include_bytes!("../../res/splash_jam.map")).as_u16_slice().iter().enumerate() {
            let x = index % 30;
            let y = index / 30;
            log::info!("{}, {}", x, y);
            TEXT_SCREENBLOCKS.get_frame(8).unwrap().get_row(y).unwrap().get(x).unwrap().write(TextEntry::new().with_tile(*tile));
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
