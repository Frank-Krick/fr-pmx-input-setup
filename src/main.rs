use application::App;
use iced::Application;

mod application;

fn main() -> std::result::Result<(), iced::Error> {
    App::run(iced::Settings::default())
}
