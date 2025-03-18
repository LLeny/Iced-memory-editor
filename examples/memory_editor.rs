use iced::{Element, Theme};
use iced_memory_editor::memory_editor::{memory_editor, Message};
use rand::Rng;

pub fn main() -> iced::Result {
    iced::application("Memory Editor - Iced", Example::update, Example::view)
        .theme(|_| Theme::Dark)
        .run()
}

struct Example {
    data: [u8; 65536],
}

impl Example {
    fn new() -> Self {
        let mut data = [0u8; 65536];
        let mut rng = rand::rng();
        data.iter_mut().for_each(|byte| {
            *byte = rng.random();
        });
        Example { data }
    }

    fn update(&mut self, _message: Message) {}

    fn view(&self) -> Element<Message> {
        let read_fn = |src: &[u8; 65536], addr: usize| src.get(addr).copied();
        memory_editor(&self.data, read_fn).into()
    }
}

impl Default for Example {
    fn default() -> Self {
        Self::new()
    }
}
