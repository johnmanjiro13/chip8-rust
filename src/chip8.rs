use iced::time::every;
use iced::{executor, Application, Clipboard, Command, Element, Subscription};
use log::debug;
use std::time::{Duration, Instant};

use crate::display::Display;

pub struct Chip8 {
    display: Display,
}

#[derive(Debug)]
pub enum Message {
    Display,
    Clock(Instant),
}

pub struct Flags {
    pub rom: Vec<u8>,
}

impl Application for Chip8 {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        debug!("flags rom {:?}", flags.rom);
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
