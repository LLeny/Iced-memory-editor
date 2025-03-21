use iced::{Element, Theme};
use iced_memory_editor::{
    memory,
    memory_editor::{memory_editor, Content, Message},
    options::MemoryEditorOptions,
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
    range: Range<usize>,
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
            range: Range { start: 0, end: 0 },
            options: MemoryEditorOptions::default(),
        }
    }
}

impl memory::MemoryEditorContext for ExampleData {
    fn perform(&mut self, action: memory::Action) {
        match action {
            memory::Action::DataUpdate(range) => self.range = range,
            memory::Action::ShowASCIIUpdate(show) => self.options.show_ascii = show,
            memory::Action::PreviewFormatUpdate(preview_data_format) => {
                self.options.preview_data_format = preview_data_format
            }
            memory::Action::RowLengthUpdate(len) => self.options.row_length = len,
        }
    }

    fn data(&self) -> Vec<u8> {
        self.data[self.range.clone()].into()
    }

    fn len(&self) -> usize {
        self.range.len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn options(&self) -> MemoryEditorOptions {
        self.options.clone()
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

    fn update(&mut self, message: Message) {
        if let Message::ActionPerformed(action) = message {
            self.content.perform(action)
        }
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
