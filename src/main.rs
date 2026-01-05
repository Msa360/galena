use galena::app::SdrApp;

pub fn main() -> iced::Result {
    iced::application("Galena", SdrApp::update, SdrApp::view)
        .subscription(SdrApp::subscription)
        .run()
}
