mod game;
mod title;

pub use game::Game;
pub use title::Title;

pub enum Screen {
    Title(Title),
    Game(Game),
}

impl Screen {
    // To be run continually in a loop.
    pub fn run(&mut self) {
        if let Some(new_screen) = match self {
            Self::Title(title) => title.run(),
            Self::Game(game) => game.run(),
        } {
            *self = new_screen;
        }
    }
}

impl Default for Screen {
    fn default() -> Self {
        Self::Title(Title::new())
    }
}
