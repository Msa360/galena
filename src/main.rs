use galena::app::SdrApp;

pub fn main() -> iced::Result {
    env_logger::init();

    iced::application("Galena", SdrApp::update, SdrApp::view)
        .theme(|_| iced::Theme::Dark)
        .subscription(SdrApp::subscription)
        .run()
}
