mod app;
mod projects;
mod update;
mod utils;

use crate::app::App;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .run()
}
