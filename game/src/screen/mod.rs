mod game;
mod game_over;
mod splash;
mod title;

pub use game::Game;
pub use game_over::GameOver;
pub use splash::Splash;
pub use title::Title;

pub enum Screen {
    Splash(Splash),
    Title(Title),
    Game(Game),
    GameOver(GameOver),
}

impl Screen {
    // To be run continually in a loop.
    pub fn run(&mut self) {
        if let Some(new_screen) = match self {
            Self::Splash(splash) => splash.run(),
            Self::Title(title) => title.run(),
            Self::Game(game) => game.run(),
            Self::GameOver(game_over) => game_over.run(),
        } {
            *self = new_screen;
        }
    }
}

impl Default for Screen {
    fn default() -> Self {
        Self::Splash(Splash::new())
    }
}
