mod game;

pub use game::Game;

pub enum Screen {
    Game(Game),
}

impl Screen {
    // To be run continually in a loop.
    pub fn run(&mut self) {
        match self {
            Self::Game(game) => game.run(),
        }
    }
}
