use iced::{Element, Theme};
use iced_memory_editor::{
    context::{Content, MemoryEditorContext}, memory_editor::memory_editor, options::MemoryEditorOptions
};
use rand::Rng;
use std::ops::Range;

pub fn main() -> iced::Result {
    iced::application("Memory Editor - Iced", Example::update, Example::view)
        .theme(|_| Theme::Dark)
        .run()
}

struct ExampleData {
    data: [u8; 65536],
    options: MemoryEditorOptions,
}

impl Default for ExampleData {
    fn default() -> Self {
        let mut data = [0u8; 65536];
        let mut rng = rand::rng();
        data.iter_mut().for_each(|byte| {
            *byte = rng.random();
        });

        Self {
            data,
            options: MemoryEditorOptions::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
}

impl MemoryEditorContext for ExampleData {
    fn data(&self, range: Range<usize>) -> Vec<u8> {
        self.data[range].into()
    }

    fn options(&self) -> MemoryEditorOptions {
        self.options.clone()
    }

    fn can_write(&self, address: usize) -> bool {
        address < 0x100usize
    }

    fn write(&mut self, address: usize, value: u8) {
        if let Some(byte) = self.data.get_mut(address) {
            *byte = value;
        }
    }
    
    fn write_options(&mut self, options: MemoryEditorOptions) {
        self.options = options;
    }
}

struct Example {
    content: Content<ExampleData>,
}

impl Example {
    fn new() -> Self {
        let data = ExampleData::default();
        let content = Content::new(data);
        Example { content }
    }

    fn update(&mut self, _message: Message) {
        
    }

    fn view(&self) -> Element<Message> {
        memory_editor(&self.content).into()
    }
}

impl Default for Example {
    fn default() -> Self {
        Self::new()
    }
}
