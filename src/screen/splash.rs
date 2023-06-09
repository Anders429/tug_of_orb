use super::{Screen, Title};
use crate::{
    game,
    game::{Color, Grid, Position},
};
use core::slice;
use gba::{
    bios::VBlankIntrWait,
    mmio::{
        bg_palbank, BG2CNT, BG_PALETTE, BLDCNT, BLDY, CHARBLOCK0_8BPP, CHARBLOCK1_8BPP,
        CHARBLOCK2_8BPP, DISPCNT, KEYINPUT, TEXT_SCREENBLOCKS,
    },
    video::{
        BackgroundControl, BlendControl, ColorEffectMode, DisplayControl, TextEntry, VideoMode,
    },
    Align4,
};

pub struct Splash {
    frame_count: u16,
}

impl Splash {
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

        BG2CNT.write(BackgroundControl::new().with_screenblock(8).with_bpp8(true));
        DISPCNT.write(DisplayControl::new().with_show_bg2(true));

        // Load palette.
        for (index, bytes) in Align4(*include_bytes!("../../res/splash_jam.pal"))
            .as_u16_slice()
            .iter()
            .enumerate()
        {
            BG_PALETTE
                .get(index)
                .unwrap()
                .write(gba::video::Color(*bytes));
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
        for (index, tile) in Align4(*include_bytes!("../../res/splash_jam.map"))
            .as_u16_slice()
            .iter()
            .enumerate()
        {
            let x = index % 30;
            let y = index / 30;
            TEXT_SCREENBLOCKS
                .get_frame(8)
                .unwrap()
                .get_row(y)
                .unwrap()
                .get(x)
                .unwrap()
                .write(TextEntry::new().with_tile(*tile));
        }

        // Fade in.
        for fade in (0..31).rev() {
            VBlankIntrWait();
            BLDY.write(fade / 2);
        }

        Self { frame_count: 0 }
    }

    pub fn run(&mut self) -> Option<Screen> {
        let keys = KEYINPUT.read();
        if self.frame_count > 180 || keys.a() {
            VBlankIntrWait();

            // Fade out.
            VBlankIntrWait();
            for fade in (0..31) {
                VBlankIntrWait();
                BLDY.write(fade / 2);
            }

            Some(Screen::Title(Title::new()))
        } else {
            let keys = KEYINPUT.read();
            VBlankIntrWait();
            self.frame_count += 1;
            None
        }
    }
}
