mod launcher;
mod project;

use iced::{Font, Theme};
use launcher::component::Launcher;
use tracing_subscriber::layer::SubscriberExt;

pub fn main() -> iced::Result {
    let console_log = tracing_subscriber::fmt::Layer::new()
        .with_ansi(true)
        .with_writer(std::io::stdout);

    let subscriber = tracing_subscriber::registry().with(console_log);

    let _ = tracing::subscriber::set_global_default(subscriber);

    iced::application("GiiGa Launcher", Launcher::update, Launcher::view)
        .default_font(Font::DEFAULT)
        .theme(|_| Theme::Dark)
        .run_with(Launcher::new)
}
