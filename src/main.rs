use iced::time::every;
use iced::{executor, Application, Clipboard, Command, Element, Settings, Subscription};
use std::time::{Duration, Instant};

mod display;

use display::Display;

fn main() {
    let mut settings = Settings::default();
    settings.window.size = (display::WIDTH as u32, display::HEIGHT as u32);
    Chip8::run(settings).unwrap();
}

struct Chip8 {
    display: Display,
}

#[derive(Debug)]
enum Message {
    Display,
    Clock(Instant),
}

impl Application for Chip8 {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                display: Display::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Chip8")
    }

    fn update(
        &mut self,
        _message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        self.display.view().map(|_| Message::Display)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let clock = every(Duration::from_millis(1000 / 60)).map(Message::Clock);
        Subscription::batch([clock])
    }
}
