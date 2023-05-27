use super::{Game, Screen};
use crate::{
    game,
    game::{Color, Grid, Position},
};
use gba::{
    mmio::{BLDCNT, BLDY, KEYINPUT},
    video::{BlendControl, ColorEffectMode},
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
                .with_mode(ColorEffectMode::Brighten),
        );
        // Fade out while we set up the screen.
        BLDY.write(16);

        Self { random_seed: 0 }
    }

    pub fn run(&mut self) -> Option<Screen> {
        let keys = KEYINPUT.read();
        if keys.a() {
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
