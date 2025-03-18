use crate::options::{MemoryEditorOptions, PreviewDataFormat};
use crate::state::State;
use crate::style::{Catalog, Style};
use iced::advanced::graphics::text::Paragraph;
use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse::Cursor;
use iced::advanced::renderer::{self, Quad};
use iced::advanced::text::Paragraph as _;
use iced::advanced::widget::{self, tree::Tree, Widget};
use iced::advanced::{mouse, Shell, Text};
use iced::alignment::Vertical;
use iced::widget::text::{Alignment, LineHeight, Shaping, Wrapping};
use iced::{keyboard, Border, Element, Event, Length, Point, Rectangle, Shadow, Size};
use std::f32;
use std::marker::PhantomData;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Message {
    OptionsToggled,
    DataUpdated,
    RowLengthChanged(usize),
    DataFormatChanged(PreviewDataFormat),
    ShowAsciiToggled(bool),
}

pub struct MemoryEditor<'a, T, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
    Theme: Catalog + iced::widget::text::Catalog,
{
    data_source: &'a T,
    read_fn: fn(&T, usize) -> Option<u8>,
    write_fn: Option<fn(&T, usize, u8)>,
    options: MemoryEditorOptions,
    class: <Theme as crate::style::Catalog>::Class<'a>,
    messages: Vec<Message>,
    _renderer: PhantomData<Renderer>,
}

impl<'a, T, Theme, Renderer> MemoryEditor<'a, T, Theme, Renderer>
where
    Theme: Catalog + iced::widget::text::Catalog + 'a,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = iced::Font> + 'a,
{
    pub fn new(data_source: &'a T, read_fn: fn(&T, usize) -> Option<u8>) -> Self {
        MemoryEditor {
            read_fn,
            write_fn: None,
            messages: vec![Message::DataUpdated],
            options: MemoryEditorOptions::default(),
            data_source,
            class: <Theme as crate::style::Catalog>::default(),
            _renderer: PhantomData,
        }
    }

    pub fn data_updated(&mut self) {
        self.messages.push(Message::DataUpdated);
    }

    pub(crate) fn with_write(mut self, write_fn: fn(&T, usize, u8)) -> Self {
        self.set_write(write_fn);
        self
    }

    pub(crate) fn set_write(&mut self, write_fn: fn(&T, usize, u8)) {
        self.write_fn = Some(write_fn);
        todo!();
    }

    pub(crate) fn with_options(mut self, options: MemoryEditorOptions) -> Self {
        self.set_options(options);
        self
    }

    pub(crate) fn set_options(&mut self, options: MemoryEditorOptions) {
        self.options = options;
        todo!();
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

    fn update_data(&self, state: &mut State) {
        let start = state.start_address;
        for (i, byte) in state.data.iter_mut().enumerate() {
            *byte = (self.read_fn)(self.data_source, start + i).unwrap_or(0xff);
        }
    }

    fn handle_mouse_interaction(
        &mut self,
        state: &mut State,
        cursor: Cursor,
        bounds: Rectangle,
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
                return (
                    true,
                    Some(Message::ShowAsciiToggled(!self.options.show_ascii)),
                );
            }

            if cursor.is_over(state.bounds.prev_format) {
                self.options.previous_data_format();
                return (
                    true,
                    Some(Message::DataFormatChanged(self.options.preview_data_format)),
                );
            }

            if cursor.is_over(state.bounds.next_format) {
                self.options.next_data_format();
                return (
                    true,
                    Some(Message::DataFormatChanged(self.options.preview_data_format)),
                );
            }

            if cursor.is_over(state.bounds.prev_row_length) {
                self.options.row_length = (self.options.row_length - 8).max(8);
                return (
                    true,
                    Some(Message::RowLengthChanged(self.options.row_length)),
                );
            }

            if cursor.is_over(state.bounds.next_row_length) {
                self.options.row_length += 8;
                return (
                    true,
                    Some(Message::RowLengthChanged(self.options.row_length)),
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
        if byte_index >= self.options.row_length {
            state.selected_address = None;
            return (true, None);
        }

        let clicked_address =
            state.start_address + (row_index * self.options.row_length) + byte_index;
        state.selected_address = Some(clicked_address);
        (true, None)
    }

    fn calculate_byte_index(&self, x_in_data: f32, state: &State) -> usize {
        let total_x = x_in_data + state.dimensions.group_spacing / 2.0;
        let byte_slot_width = state.dimensions.byte_width
            + (state.dimensions.group_spacing / state.dimensions.group_char_len as f32);

        (total_x / byte_slot_width) as usize
    }

    fn background(&self, renderer: &mut Renderer, style: &Style, bounds: Rectangle)
    where
        Renderer: renderer::Renderer,
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

    fn separator(&self, renderer: &mut Renderer, style: &Style, bounds: Rectangle, x: f32)
    where
        Renderer: renderer::Renderer,
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

    fn row(
        &self,
        renderer: &mut Renderer,
        style: &Style,
        bounds: Rectangle,
        state: &State,
        addr: &usize,
        row_data: &[u8],
    ) where
        Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
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

        if self.options.show_ascii {
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
                if selected_addr >= *addr && selected_addr < addr + self.options.row_length {
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

    fn bottom_panel(
        &self,
        _tree: &widget::Tree,
        renderer: &mut Renderer,
        state: &State,
        style: &Style,
        bounds: Rectangle,
    ) where
        Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
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

        // Options text
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

        // Jump to text
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
                    < state.start_address + state.dimensions.row_count * self.options.row_length
            {
                let required_bytes = match self.options.preview_data_format {
                    PreviewDataFormat::U8 | PreviewDataFormat::I8 => 1,
                    PreviewDataFormat::U16 | PreviewDataFormat::I16 => 2,
                    PreviewDataFormat::U32 | PreviewDataFormat::I32 | PreviewDataFormat::F32 => 4,
                    PreviewDataFormat::U64 | PreviewDataFormat::I64 | PreviewDataFormat::F64 => 8,
                };

                let mut preview_data = [0u8; 8];
                if let Some(data_slice) = state.data.get(
                    selected_addr - state.start_address
                        ..selected_addr - state.start_address + required_bytes,
                ) {
                    preview_data[..required_bytes].copy_from_slice(data_slice);
                }

                let value_text = self.format_preview_value(
                    &preview_data[..required_bytes],
                    &self.options.preview_data_format,
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

    fn options_panel(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        layout: Layout<'_>,
    ) where
        Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = iced::Font>,
    {
        let state = tree.state.downcast_ref::<State>();

        if !state.options_open {
            return;
        }

        let style = <Theme as Catalog>::style(theme, &self.class);
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

        // Draw row length label
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

        // Draw left arrow button to decrease row length
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

        // Draw row length value
        renderer.fill_text(
            Text {
                content: format!("{}", self.options.row_length),
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

        // Draw right arrow button to increase row length
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

        // Draw preview data label
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

        // Draw preview data format
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
                content: format!("{}", self.options.preview_data_format),
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

        // Draw show ASCII checkbox
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

        if self.options.show_ascii {
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

impl<'a, Theme, Renderer, T> Widget<Message, Theme, Renderer>
    for MemoryEditor<'a, T, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = iced::Font> + 'a,
    Theme: Catalog + iced::widget::text::Catalog + 'a,
    T: 'a + Sized,
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
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();

        let text_size = renderer.default_size();
        let text_line_height = LineHeight::default();

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

        state.dimensions.byte_width = state.dimensions.char_width * 2.5;
        state.dimensions.group_spacing = state.dimensions.char_width;
        state.dimensions.section_separator_spacing = state.dimensions.char_width * 2.0;

        state.dimensions.section_data_start = state.dimensions.char_width
            * state.dimensions.address_char_len as f32
            + state.dimensions.section_separator_spacing;
        state.dimensions.section_ascii_start = state.dimensions.section_data_start
            + state.dimensions.byte_width * self.options.row_length as f32
            + (self.options.row_length as f32 / state.dimensions.group_char_len as f32 - 1.0)
                * state.dimensions.group_spacing
            + state.dimensions.section_separator_spacing;

        state.dimensions.address_separator_x =
            state.dimensions.section_data_start - state.dimensions.section_separator_spacing / 2.0;
        state.dimensions.ascii_separator_x =
            state.dimensions.section_ascii_start - state.dimensions.section_separator_spacing / 2.0;
        state.text.jumpto_len = state.text.jumpto_text.len() as f32 * state.dimensions.char_width;

        let total_bytes = state.dimensions.row_count * self.options.row_length;
        if state.data.len() != total_bytes {
            state.data.resize(total_bytes, 0);
        }

        self.update_data(state);

        let options_text = "Options";
        let options_width = options_text.len() as f32 * state.dimensions.char_width;
        state.bounds.options = Rectangle {
            x: limits.min().width,
            y: limits.max().height - state.dimensions.char_height * 1.5,
            width: options_width,
            height: state.dimensions.char_height * 1.5,
        };

        let total_width = limits.max().width;
        let jumpto_x = (total_width - state.text.jumpto_len - options_width) / 2.0;
        let input_x = jumpto_x + state.text.jumpto_len + state.dimensions.char_width;

        state.bounds.addr_input = Rectangle {
            x: input_x,
            y: limits.max().height - state.dimensions.char_height * 1.3,
            width: (state.dimensions.char_width + 1.0)
                * state.dimensions.address_char_len as f32
                * 1.1,
            height: state.dimensions.char_height * 1.1,
        };

        let panel_bounds = Rectangle {
            x: state.dimensions.char_width * 0.5,
            y: limits.max().height
                - state.dimensions.char_height * 1.5
                - state.dimensions.char_height * 4.0,
            width: limits.max().width - state.dimensions.char_width,
            height: state.dimensions.char_height * 4.0,
        };

        let label_width = 120.0;
        let offset_y = panel_bounds.y + state.dimensions.char_height * 0.5;
        let checkbox_size = state.dimensions.char_height * 0.8;
        let base_x = panel_bounds.x + state.dimensions.char_width + label_width;

        state.bounds.show_ascii_checkbox = Rectangle {
            x: base_x + 3.0 * state.dimensions.char_width,
            y: offset_y + state.dimensions.char_height * 2.0,
            width: checkbox_size,
            height: checkbox_size,
        };

        state.bounds.text_format = Rectangle {
            x: base_x + 2.0 * state.dimensions.char_width,
            y: offset_y + state.dimensions.char_height,
            width: state.dimensions.char_width * 3.0,
            height: state.dimensions.char_height,
        };

        state.bounds.prev_format = Rectangle {
            x: base_x,
            y: offset_y + state.dimensions.char_height,
            width: state.dimensions.char_width,
            height: state.dimensions.char_height,
        };

        state.bounds.next_format = Rectangle {
            x: base_x + 6.0 * state.dimensions.char_width,
            y: offset_y + state.dimensions.char_height,
            width: state.dimensions.char_width,
            height: state.dimensions.char_height,
        };

        state.bounds.prev_row_length = Rectangle {
            x: base_x,
            y: offset_y,
            width: state.dimensions.char_width,
            height: state.dimensions.char_height,
        };

        state.bounds.next_row_length = Rectangle {
            x: base_x + 6.0 * state.dimensions.char_width,
            y: offset_y,
            width: state.dimensions.char_width,
            height: state.dimensions.char_height,
        };

        state.bounds.text_row_length = Rectangle {
            x: base_x + 2.0 * state.dimensions.char_width,
            y: offset_y,
            width: state.dimensions.char_width * 3.0,
            height: state.dimensions.char_height,
        };

        layout::Node::with_children(limits.max(), vec![])
    }

    fn diff(&self, _tree: &mut Tree) {}

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &Rectangle,
    ) where
        Theme: Catalog,
    {
        let state = tree.state.downcast_ref::<State>();
        let style = <Theme as crate::style::Catalog>::style(theme, &self.class);
        let bounds = layout.bounds();

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

        if self.options.show_ascii {
            self.separator(
                renderer,
                &style,
                separator_bounds,
                state.dimensions.ascii_separator_x,
            );
        }

        let mut addr = state.start_address;
        let mut y_offset = bounds.y;

        for row_i in 0..state.dimensions.row_count {
            let row_start = row_i * self.options.row_length;
            let row_end = row_start + self.options.row_length;
            let row_slice = &state.data[row_start..row_end];

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
                row_slice,
            );

            addr += self.options.row_length;
            y_offset += state.dimensions.char_height;
        }

        self.bottom_panel(tree, renderer, state, &style, bounds);
        self.options_panel(tree, renderer, theme, layout);
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
        _renderer: &Renderer,
        _operation: &mut dyn widget::Operation,
    ) {
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();

        let local_shell = Shell::new(&mut self.messages);

        if local_shell.is_event_captured() {
            shell.capture_event();
        }

        shell.request_redraw_at(local_shell.redraw_request());
        shell.request_input_method(local_shell.input_method());

        self.messages
            .clone()
            .iter()
            .for_each(|message| match message {
                Message::OptionsToggled => {
                    state.options_open = !state.options_open;
                    shell.invalidate_layout();
                    shell.request_redraw();
                }
                Message::DataUpdated => {
                    self.update_data(state);
                    shell.request_redraw();
                }
                Message::RowLengthChanged(len) => {
                    self.options.row_length = *len;
                    shell.invalidate_layout();
                    shell.request_redraw();
                }
                Message::DataFormatChanged(format) => {
                    self.options.preview_data_format = *format;
                    shell.request_redraw();
                }
                Message::ShowAsciiToggled(show) => {
                    self.options.show_ascii = *show;
                    shell.invalidate_layout();
                    shell.request_redraw();
                }
            });

        self.messages.clear();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let (true, message) = self.handle_mouse_interaction(state, cursor, bounds) {
                    if let Some(msg) = message {
                        self.messages.push(msg);
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
                let step = y.trunc() * self.options.row_length as f32;
                if step.is_sign_negative() {
                    state.start_address = state.start_address.saturating_sub(step.abs() as usize);
                } else {
                    state.start_address += step as usize;
                }
                self.data_updated();
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
        _renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        mouse::Interaction::default()
    }
}

impl<'a, Theme, Renderer, T> From<MemoryEditor<'a, T, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = iced::Font> + 'a,
    Theme: Catalog + iced::widget::text::Catalog + 'a,
    T: 'a + Sized,
{
    fn from(memory_editor: MemoryEditor<'a, T, Theme, Renderer>) -> Self {
        Self::new(memory_editor)
    }
}

pub fn memory_editor<'a, T, Theme, Renderer>(
    data_source: &'a T,
    read_fn: fn(&T, usize) -> Option<u8>,
) -> MemoryEditor<'a, T, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer<Font = iced::Font> + 'a,
    Theme: Catalog + iced::widget::text::Catalog + 'a,
{
    MemoryEditor::new(data_source, read_fn)
}
