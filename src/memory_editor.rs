use crate::context::{self, Action};
use crate::options::{MemoryEditorOptions, PreviewDataFormat};
use crate::state::State;
use crate::style::{Catalog, Style};
use iced::advanced::graphics::text::Paragraph;
use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse::Cursor;
use iced::advanced::renderer::{self, Quad};
use iced::advanced::text::Paragraph as _;
use iced::advanced::widget::{self, tree::Tree, Widget};
use iced::advanced::{mouse, Text};
use iced::alignment::Vertical;
use iced::widget::text::{Alignment, LineHeight, Shaping, Wrapping};
use iced::{keyboard, Border, Element, Event, Length, Point, Rectangle, Shadow, Size};
use std::cell::RefCell;
use std::f32;
use std::ops::Range;

#[derive(Clone, PartialEq, Debug)]
pub enum Message {
    ActionPerformed(Action),
    RowLengthChanged(usize),
    DataFormatChanged(PreviewDataFormat),
    ShowAsciiToggled(bool),
    OptionsToggled,
}

pub struct Content<M: context::MemoryEditorContext>(RefCell<Internal<M>>);

struct Internal<M>
where
    M: context::MemoryEditorContext,
{
    context: M,
    is_dirty: bool,
}

impl<M> Content<M>
where
    M: context::MemoryEditorContext,
{
    pub fn new(context: M) -> Self {
        Self(RefCell::new(Internal {
            context,
            is_dirty: true,
        }))
    }

    pub fn perform(&mut self, action: Action) {
        let internal = self.0.get_mut();
        internal.context.perform(action);
        internal.is_dirty = false;
    }
}

pub struct MemoryEditor<'a, T, M>
where
    T: Catalog + iced::widget::text::Catalog,
    M: context::MemoryEditorContext,
{
    content: &'a Content<M>,
    class: <T as crate::style::Catalog>::Class<'a>,
}

impl<'a, T, M> MemoryEditor<'a, T, M>
where
    T: Catalog + iced::widget::text::Catalog + 'a,
    M: context::MemoryEditorContext,
{
    pub fn new(content: &'a Content<M>) -> Self {
        MemoryEditor {
            class: <T as crate::style::Catalog>::default(),
            content,
        }
    }

    fn format_preview_value(&self, data: &[u8], format: &PreviewDataFormat) -> String {
        if data.is_empty() {
            return String::from("#Error#");
        }

        match format {
            PreviewDataFormat::U8 => bytemuck::from_bytes::<u8>(data).to_string(),
            PreviewDataFormat::U16 => bytemuck::from_bytes::<u16>(data).to_string(),
            PreviewDataFormat::U32 => bytemuck::from_bytes::<u32>(data).to_string(),
            PreviewDataFormat::U64 => bytemuck::from_bytes::<u64>(data).to_string(),
            PreviewDataFormat::I8 => bytemuck::from_bytes::<i8>(data).to_string(),
            PreviewDataFormat::I16 => bytemuck::from_bytes::<i16>(data).to_string(),
            PreviewDataFormat::I32 => bytemuck::from_bytes::<i32>(data).to_string(),
            PreviewDataFormat::I64 => bytemuck::from_bytes::<i64>(data).to_string(),
            PreviewDataFormat::F32 => format!("{:.3}", bytemuck::from_bytes::<f32>(data)), // TODO: fix
            PreviewDataFormat::F64 => format!("{:.3}", bytemuck::from_bytes::<f64>(data)), // TODO: fix
        }
    }

    fn handle_mouse_interaction(
        &mut self,
        state: &mut State,
        cursor: Cursor,
        bounds: Rectangle,
        options: &MemoryEditorOptions,
    ) -> (bool, Option<Message>) {
        if state.addr_input.focused {
            state.addr_input.focused = false;
        }

        if cursor.position().is_none() {
            return (false, None);
        }

        let position = cursor.position().unwrap();

        if cursor.is_over(state.bounds.options) {
            return (true, Some(Message::OptionsToggled));
        }

        if cursor.is_over(state.bounds.addr_input) {
            state.addr_input.focused = true;
            state.addr_input.value.clear();
            return (true, None);
        }

        if state.options_open {
            if cursor.is_over(state.bounds.show_ascii_checkbox) {
                return (true, Some(Message::ShowAsciiToggled(!options.show_ascii)));
            }

            if cursor.is_over(state.bounds.prev_format) {
                return (
                    true,
                    Some(Message::DataFormatChanged(options.previous_data_format())),
                );
            }

            if cursor.is_over(state.bounds.next_format) {
                return (
                    true,
                    Some(Message::DataFormatChanged(options.next_data_format())),
                );
            }

            if cursor.is_over(state.bounds.prev_row_length) {
                return (
                    true,
                    Some(Message::RowLengthChanged((options.row_length - 8).max(8))),
                );
            }

            if cursor.is_over(state.bounds.next_row_length) {
                return (
                    true,
                    Some(Message::RowLengthChanged(options.row_length + 8)),
                );
            }
        }

        let row_index = ((position.y - bounds.y) / state.dimensions.char_height).trunc() as usize;
        if row_index >= state.dimensions.row_count {
            return (false, None);
        }

        let x_in_data = position.x - (bounds.x + state.dimensions.section_data_start);
        if x_in_data < 0.0 {
            return (false, None);
        }

        let byte_index = self.calculate_byte_index(x_in_data, state);
        if byte_index >= options.row_length {
            state.selected_address = None;
            return (true, None);
        }

        let clicked_address = state.start_address + (row_index * options.row_length) + byte_index;
        state.selected_address = Some(clicked_address);
        (true, None)
    }

    fn calculate_byte_index(&self, x_in_data: f32, state: &State) -> usize {
        let total_x = x_in_data + state.dimensions.group_spacing / 2.0;
        let byte_slot_width = state.dimensions.byte_width
            + (state.dimensions.group_spacing / state.dimensions.group_char_len as f32);

        (total_x / byte_slot_width) as usize
    }

    fn background<R>(&self, renderer: &mut R, style: &Style, bounds: Rectangle)
    where
        R: renderer::Renderer,
    {
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                shadow: style.shadow,
            },
            style.background,
        );
    }

    fn separator<R>(&self, renderer: &mut R, style: &Style, bounds: Rectangle, x: f32)
    where
        R: renderer::Renderer,
    {
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x,
                    y: bounds.y + 2.0,
                    width: 1.0,
                    height: bounds.height - 4.0,
                },
                border: style.border,
                shadow: style.shadow,
            },
            style.text_color,
        );
    }

    fn row<R>(
        &self,
        renderer: &mut R,
        style: &Style,
        bounds: Rectangle,
        state: &State,
        addr: &usize,
        row_data: &[u8],
        options: &MemoryEditorOptions,
    ) where
        R: renderer::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
    {
        renderer.fill_text(
            Text {
                content: format!("{:06X}", addr),
                bounds: Size::new(
                    state.dimensions.char_width * state.dimensions.address_char_len as f32,
                    bounds.height,
                ),
                ..state.text_defaults
            },
            Point::new(bounds.x, bounds.y),
            style.inactive_text_color,
            bounds,
        );

        let mut x_offset = bounds.x + state.dimensions.section_data_start;

        for (byte_idx, byte) in row_data.iter().enumerate() {
            let byte_addr = addr + byte_idx;
            let is_selected = state.selected_address == Some(byte_addr);

            if is_selected {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: x_offset,
                            y: bounds.y,
                            width: state.dimensions.char_width * 2.0,
                            height: bounds.height,
                        },
                        border: Border::default(),
                        shadow: style.shadow,
                    },
                    style.selection_color,
                );
            }

            renderer.fill_text(
                Text {
                    content: format!("{:02x}", byte),
                    bounds: Size::new(state.dimensions.char_width * 2.0, bounds.height),
                    ..state.text_defaults
                },
                Point::new(x_offset, bounds.y),
                if is_selected {
                    style.selected_text_color
                } else {
                    style.text_color
                },
                bounds,
            );

            x_offset += state.dimensions.byte_width;
            if (byte_idx + 1) % state.dimensions.group_char_len == 0 {
                x_offset += state.dimensions.group_spacing;
            }
        }

        if options.show_ascii {
            x_offset = state.dimensions.section_ascii_start;

            let ascii_string: String = row_data
                .iter()
                .map(|&byte| {
                    if (32..=126).contains(&byte) {
                        byte as char
                    } else {
                        '.'
                    }
                })
                .collect();

            let string_width = ascii_string.len() as f32 * state.dimensions.char_width;

            renderer.fill_text(
                Text {
                    content: ascii_string,
                    bounds: Size::new(string_width, bounds.height),
                    ..state.text_defaults
                },
                Point::new(x_offset, bounds.y),
                style.inactive_text_color,
                bounds,
            );

            if let Some(selected_addr) = state.selected_address {
                if selected_addr >= *addr && selected_addr < addr + options.row_length {
                    let ascii_x = state.dimensions.section_ascii_start
                        + ((selected_addr - addr) as f32 * state.dimensions.char_width);

                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: ascii_x,
                                y: bounds.y,
                                width: state.dimensions.char_width,
                                height: bounds.height,
                            },
                            border: style.border,
                            shadow: style.shadow,
                        },
                        style.selection_color,
                    );
                }
            }
        }
    }

    fn bottom_panel<R>(
        &self,
        _tree: &widget::Tree,
        renderer: &mut R,
        state: &State,
        style: &Style,
        bounds: Rectangle,
        options: &MemoryEditorOptions,
    ) where
        R: renderer::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
    {
        let panel_bounds = Rectangle {
            y: bounds.y + bounds.height - state.dimensions.char_height * 1.5,
            height: state.dimensions.char_height * 1.5,
            ..bounds
        };

        renderer.fill_quad(
            Quad {
                bounds: panel_bounds,
                border: Border {
                    width: 1.0,
                    ..style.border
                },
                shadow: Shadow::default(),
            },
            style.background,
        );

        let panel_options_bounds = Rectangle {
            x: panel_bounds.x,
            y: panel_bounds.y,
            width: state.bounds.options.width,
            height: panel_bounds.height,
        };

        renderer.fill_text(
            Text {
                content: "Options".to_string(),
                bounds: Size::new(panel_options_bounds.width, panel_bounds.height),
                ..state.text_defaults
            },
            Point::new(
                panel_options_bounds.x,
                panel_bounds.y + state.dimensions.char_width / 2.0,
            ),
            style.text_color,
            panel_options_bounds,
        );

        let jumpto_bounds = Rectangle {
            x: state.bounds.addr_input.x - state.text.jumpto_len - state.dimensions.char_width,
            y: panel_bounds.y,
            width: state.text.jumpto_len,
            height: panel_bounds.height,
        };

        renderer.fill_text(
            Text {
                content: state.text.jumpto_text.clone(),
                bounds: Size::new(state.text.jumpto_len, panel_bounds.height),
                ..state.text_defaults
            },
            Point::new(
                jumpto_bounds.x,
                panel_bounds.y + state.dimensions.char_width / 2.0,
            ),
            style.text_color,
            jumpto_bounds,
        );

        let panel_input_bounds = state.bounds.addr_input;

        renderer.fill_quad(
            Quad {
                bounds: panel_input_bounds,
                border: Border {
                    width: 1.0,
                    color: if state.addr_input.focused {
                        style.text_color
                    } else {
                        style.border.color
                    },
                    ..style.border
                },
                shadow: Shadow::default(),
            },
            style.primary_color,
        );

        renderer.fill_text(
            Text {
                content: state.addr_input.value.clone(),
                bounds: Size::new(
                    panel_input_bounds.width - state.dimensions.char_width,
                    panel_input_bounds.height,
                ),
                ..state.text_defaults
            },
            Point::new(
                panel_input_bounds.x + state.dimensions.char_width / 2.0,
                panel_bounds.y + state.dimensions.char_width / 2.0,
            ),
            style.text_color,
            panel_input_bounds,
        );

        if let Some(selected_addr) = state.selected_address {
            if selected_addr >= state.start_address
                && selected_addr
                    < state.start_address + state.dimensions.row_count * options.row_length
            {
                let required_bytes = match options.preview_data_format {
                    PreviewDataFormat::U8 | PreviewDataFormat::I8 => 1,
                    PreviewDataFormat::U16 | PreviewDataFormat::I16 => 2,
                    PreviewDataFormat::U32 | PreviewDataFormat::I32 | PreviewDataFormat::F32 => 4,
                    PreviewDataFormat::U64 | PreviewDataFormat::I64 | PreviewDataFormat::F64 => 8,
                };

                let mut preview_data = [0u8; 8];
                if let Some(data_slice) = self.content.0.borrow().context.data().get(
                    selected_addr - state.start_address
                        ..selected_addr - state.start_address + required_bytes,
                ) {
                    preview_data[..required_bytes].copy_from_slice(data_slice);
                }

                let value_text = self.format_preview_value(
                    &preview_data[..required_bytes],
                    &options.preview_data_format,
                );

                let value_width = value_text.len() as f32 * state.dimensions.char_width;
                let value_bound = Rectangle {
                    x: panel_bounds.x + panel_bounds.width
                        - value_width
                        - state.dimensions.char_width,
                    y: panel_bounds.y,
                    width: value_width,
                    height: panel_bounds.height,
                };

                renderer.fill_text(
                    Text {
                        content: value_text,
                        bounds: Size::new(value_width, panel_bounds.height),
                        ..state.text_defaults
                    },
                    Point::new(
                        value_bound.x,
                        panel_bounds.y + state.dimensions.char_width / 2.0,
                    ),
                    style.text_color,
                    value_bound,
                );
            }
        }
    }

    fn options(&self) -> MemoryEditorOptions {
        self.content.0.borrow().context.options()
    }

    fn action_performed(&self, action: Action) -> Message {
        Message::ActionPerformed(action)
    }

    fn data_update_message(&self, state: &State, len: usize) -> Message {
        self.action_performed(Action::DataUpdate(Range::<usize> {
            start: state.start_address,
            end: state.start_address + state.dimensions.row_count * len,
        }))
    }

    fn options_panel<R>(
        &self,
        tree: &widget::Tree,
        renderer: &mut R,
        theme: &T,
        layout: Layout<'_>,
        options: &MemoryEditorOptions,
    ) where
        R: renderer::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
    {
        let state = tree.state.downcast_ref::<State>();

        if !state.options_open {
            return;
        }

        let style = <T as Catalog>::style(theme, &self.class);
        let bounds = layout.bounds();

        let panel_bounds = Rectangle {
            y: bounds.y + bounds.height
                - state.dimensions.char_height * 1.5
                - state.dimensions.char_height * 4.0,
            height: state.dimensions.char_height * 4.0,
            ..bounds
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: panel_bounds,
                border: style.border,
                shadow: style.shadow,
            },
            style.primary_color,
        );

        let label_width = 120.0;
        let offset_y = panel_bounds.y + state.dimensions.char_height * 0.5;

        renderer.fill_text(
            Text {
                content: "Row length".into(),
                bounds: Size::new(label_width, state.dimensions.char_height),
                ..state.text_defaults
            },
            Point::new(panel_bounds.x + state.dimensions.char_width, offset_y),
            style.text_color,
            panel_bounds,
        );

        renderer.fill_text(
            Text {
                content: "<".into(),
                bounds: Size::new(state.dimensions.char_width, state.dimensions.char_height),
                ..state.text_defaults
            },
            Point::new(
                state.bounds.prev_row_length.x,
                state.bounds.prev_row_length.y,
            ),
            style.text_color,
            state.bounds.prev_row_length,
        );

        renderer.fill_text(
            Text {
                content: format!("{}", options.row_length),
                bounds: Size::new(
                    state.dimensions.char_width * 3.0,
                    state.dimensions.char_height,
                ),
                ..state.text_defaults
            },
            Point::new(
                state.bounds.text_row_length.x,
                state.bounds.text_row_length.y,
            ),
            style.text_color,
            state.bounds.text_row_length,
        );

        renderer.fill_text(
            Text {
                content: ">".into(),
                bounds: Size::new(state.dimensions.char_width, state.dimensions.char_height),
                ..state.text_defaults
            },
            Point::new(
                state.bounds.next_row_length.x,
                state.bounds.next_row_length.y,
            ),
            style.text_color,
            state.bounds.next_row_length,
        );

        renderer.fill_text(
            Text {
                content: "Preview data".into(),
                bounds: Size::new(label_width, state.dimensions.char_height),
                ..state.text_defaults
            },
            Point::new(
                panel_bounds.x + state.dimensions.char_width,
                offset_y + state.dimensions.char_height,
            ),
            style.text_color,
            panel_bounds,
        );

        renderer.fill_text(
            Text {
                content: "<".into(),
                bounds: Size::new(state.dimensions.char_width, state.dimensions.char_height),
                ..state.text_defaults
            },
            Point::new(state.bounds.prev_format.x, state.bounds.prev_format.y),
            style.text_color,
            state.bounds.prev_format,
        );

        renderer.fill_text(
            Text {
                content: format!("{}", options.preview_data_format),
                bounds: Size::new(
                    state.dimensions.char_width * 3.0,
                    state.dimensions.char_height,
                ),
                ..state.text_defaults
            },
            Point::new(state.bounds.text_format.x, state.bounds.text_format.y),
            style.text_color,
            state.bounds.text_format,
        );

        renderer.fill_text(
            Text {
                content: ">".into(),
                bounds: Size::new(state.dimensions.char_width, state.dimensions.char_height),
                ..state.text_defaults
            },
            Point::new(state.bounds.next_format.x, state.bounds.next_format.y),
            style.text_color,
            state.bounds.next_format,
        );

        renderer.fill_text(
            Text {
                content: "Show ASCII".into(),
                bounds: Size::new(label_width, state.dimensions.char_height),
                ..state.text_defaults
            },
            Point::new(
                panel_bounds.x + state.dimensions.char_width,
                state.bounds.show_ascii_checkbox.y,
            ),
            style.text_color,
            panel_bounds,
        );

        let checkbox_size = state.dimensions.char_height * 0.8;
        let checkbox_bounds = state.bounds.show_ascii_checkbox;

        renderer.fill_quad(
            renderer::Quad {
                bounds: checkbox_bounds,
                border: Border {
                    width: 1.0,
                    color: style.text_color,
                    radius: (2.0).into(),
                },
                shadow: Shadow::default(),
            },
            style.background,
        );

        if options.show_ascii {
            let padding = checkbox_size * 0.2;
            let inner_bounds = Rectangle {
                x: checkbox_bounds.x + padding,
                y: checkbox_bounds.y + padding,
                width: checkbox_bounds.width - (padding * 2.0),
                height: checkbox_bounds.height - (padding * 2.0),
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds: inner_bounds,
                    border: Border {
                        radius: (1.0).into(),
                        ..Border::default()
                    },
                    shadow: Shadow::default(),
                },
                style.text_color,
            );
        }
    }
}

impl<'a, T, R, M> Widget<Message, T, R> for MemoryEditor<'a, T, M>
where
    R: renderer::Renderer + iced::advanced::text::Renderer<Font = iced::Font> + 'a,
    T: Catalog + iced::widget::text::Catalog + 'a,
    M: context::MemoryEditorContext,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Fill,
        }
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &R,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        let text_size = renderer.default_size();
        let text_line_height = LineHeight::default();
        let options = self.options();

        state.dimensions.char_height = text_line_height.to_absolute(text_size).into();
        state.dimensions.row_count = (limits.max().height / state.dimensions.char_height).floor()
            as usize
            - 1
            - if state.options_open { 4 } else { 0 };

        state.text_defaults.line_height = LineHeight::default();

        state.dimensions.char_width = Paragraph::with_text(Text {
            content: "0",
            bounds: Size::INFINITY,
            size: renderer.default_size(),
            line_height: LineHeight::default(),
            font: iced::Font::MONOSPACE,
            align_x: Alignment::Left,
            align_y: Vertical::Top,
            shaping: Shaping::Advanced,
            wrapping: Wrapping::None,
        })
        .min_bounds()
        .width;

        state.update_dimensions(options.row_length as f32);
        state.update_bounds(limits);

        layout::Node::with_children(limits.max(), vec![])
    }

    fn diff(&self, _tree: &mut Tree) {}

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut R,
        theme: &T,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &Rectangle,
    ) where
        T: Catalog,
    {
        let state = tree.state.downcast_ref::<State>();
        let style = <T as crate::style::Catalog>::style(theme, &self.class);
        let bounds = layout.bounds();
        let options = self.options();

        self.background(renderer, &style, bounds);

        let separator_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y,
            height: state.dimensions.row_count as f32 * state.dimensions.char_height,
            width: bounds.width,
        };

        self.separator(
            renderer,
            &style,
            separator_bounds,
            state.dimensions.address_separator_x,
        );

        if options.show_ascii {
            self.separator(
                renderer,
                &style,
                separator_bounds,
                state.dimensions.ascii_separator_x,
            );
        }

        let internal = self.content.0.borrow();

        if !internal.is_dirty {
            let mut addr = state.start_address;
            let mut y_offset = bounds.y;

            for slice in internal
                .context
                .data()
                .chunks_exact(options.row_length)
                .take(state.dimensions.row_count)
            {
                self.row(
                    renderer,
                    &style,
                    Rectangle {
                        x: bounds.x,
                        y: y_offset,
                        width: bounds.width,
                        height: state.dimensions.char_height,
                    },
                    state,
                    &addr,
                    slice,
                    &options,
                );

                addr += options.row_length;
                y_offset += state.dimensions.char_height;
            }
        }

        self.bottom_panel(tree, renderer, state, &style, bounds, &options);
        self.options_panel(tree, renderer, theme, layout, &options);
    }

    fn tag(&self) -> widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn children(&self) -> Vec<widget::Tree> {
        vec![]
    }

    fn operate(
        &self,
        _state: &mut widget::Tree,
        _layout: Layout<'_>,
        _renderer: &R,
        _operation: &mut dyn widget::Operation,
    ) {
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &R,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let options = self.options();

        let mut publish = |msg| shell.publish(msg);

        if self.content.0.borrow().context.is_empty() {
            publish(self.data_update_message(state, options.row_length));
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let (true, message) =
                    self.handle_mouse_interaction(state, cursor, bounds, &options)
                {
                    if let Some(message) = message {
                        match message {
                            Message::OptionsToggled => {
                                state.options_open = !state.options_open;
                                shell.invalidate_layout();
                            }
                            Message::RowLengthChanged(len) => {
                                publish(self.action_performed(Action::RowLengthUpdate(len)));
                                publish(self.data_update_message(state, len));
                                shell.invalidate_layout();
                            }
                            Message::DataFormatChanged(format) => {
                                publish(self.action_performed(Action::PreviewFormatUpdate(format)));
                            }
                            Message::ShowAsciiToggled(show) => {
                                publish(self.action_performed(Action::ShowASCIIUpdate(show)));
                                shell.invalidate_layout();
                            }
                            _ => (),
                        };
                    }
                    shell.request_redraw();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                if state.addr_input.focused =>
            {
                match key {
                    keyboard::Key::Character(c) => {
                        if state.addr_input.value.len() < state.dimensions.address_char_len
                            && c.as_str()
                                .chars()
                                .next()
                                .is_some_and(|ch| ch.is_ascii_hexdigit())
                        {
                            if let Some(ch) = c.as_str().chars().next() {
                                state.addr_input.value.push(ch);
                                shell.request_redraw();
                            }
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                        state.addr_input.value.pop();
                        shell.request_redraw();
                    }
                    keyboard::Key::Named(keyboard::key::Named::Enter) => {
                        if let Ok(addr) = usize::from_str_radix(&state.addr_input.value, 16) {
                            state.start_address = addr;
                        }
                        state.addr_input.focused = false;
                        publish(self.data_update_message(state, options.row_length));
                        shell.request_redraw();
                    }
                    keyboard::Key::Named(keyboard::key::Named::Escape) => {
                        state.addr_input.focused = false;
                        shell.request_redraw();
                    }
                    _ => {}
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { y, .. },
            }) => {
                let step = y.trunc() * options.row_length as f32;
                state.start_address = if step.is_sign_negative() {
                    state.start_address.saturating_sub(step.abs() as usize)
                } else {
                    state.start_address + step as usize
                };
                publish(self.data_update_message(state, options.row_length));
            }
            _ => (),
        }
    }

    fn mouse_interaction(
        &self,
        _state: &widget::Tree,
        _layout: Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &R,
    ) -> iced::advanced::mouse::Interaction {
        mouse::Interaction::default()
    }
}

impl<'a, T, R, M> From<MemoryEditor<'a, T, M>> for Element<'a, Message, T, R>
where
    R: renderer::Renderer + iced::advanced::text::Renderer<Font = iced::Font> + 'a,
    T: Catalog + iced::widget::text::Catalog + 'a,
    M: context::MemoryEditorContext,
{
    fn from(memory_editor: MemoryEditor<'a, T, M>) -> Self {
        Self::new(memory_editor)
    }
}

pub fn memory_editor<'a, T, M>(content: &'a Content<M>) -> MemoryEditor<'a, T, M>
where
    T: Catalog + iced::widget::text::Catalog + 'a,
    M: context::MemoryEditorContext,
{
    MemoryEditor::new(content)
}
