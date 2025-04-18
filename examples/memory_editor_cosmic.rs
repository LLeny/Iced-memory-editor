use cosmic::iced::widget::column;
use cosmic::iced::Length;
use cosmic::{app::Settings, executor, Core, Element, Task};
use iced_memory_editor::context::{Content, MemoryEditorContext};
use iced_memory_editor::memory_editor::memory_editor;
use iced_memory_editor::options::MemoryEditorOptions;
use rand::Rng;
use std::ops::Range;
use std::vec;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cosmic::app::run::<App>(Settings::default(), vec![])?;
    Ok(())
}

#[derive(Clone, Debug)]
pub enum Message {}

pub struct App {
    core: Core,
    content: Content<ExampleData>,
}

impl cosmic::Application for App {
    type Executor = executor::Default;
    type Flags = Vec<String>;
    type Message = Message;

    const APP_ID: &'static str = "org.lleny.iced_memory_editor";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _input: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let app = App {
            core,
            content: Content::new(ExampleData::default()),
        };

        (app, Task::none())
    }

    fn update(&mut self, _message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        Task::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let memory_editor = memory_editor(&self.content);
        let centered = cosmic::widget::container(column![memory_editor]).center(Length::Fill);
        Element::from(centered)
    }
}

impl App where Self: cosmic::Application {}

struct ExampleData {
    needs_refresh: bool,
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
            needs_refresh: false,
            options: MemoryEditorOptions::default(),
        }
    }
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

    fn refresh_data(&mut self) -> bool {
        if self.needs_refresh {
            self.needs_refresh = false;
            true
        } else {
            false
        }
    }
}
