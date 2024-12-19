mod launcher;
mod project;

use iced::{Font, Theme};
use launcher::component::Launcher;

pub fn main() -> iced::Result {
    iced::application("GiiGa Launcher", Launcher::update, Launcher::view)
        .default_font(Font::DEFAULT)
        .theme(|_| Theme::Dark)
        .run_with(Launcher::new)
}
