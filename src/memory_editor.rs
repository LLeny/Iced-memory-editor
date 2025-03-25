use crate::context::{self, Action, Content, MemoryEditorContext};
use crate::options::{MemoryEditorOptions, PreviewDataFormat};
use crate::state::State;
use crate::style::Style;
use std::f32;
use std::ops::Range;

#[cfg(feature = "iced")]
use crate::style::Catalog;
#[cfg(feature = "iced")]
use iced_core::{
    alignment::Vertical,
    layout::{self, Layout},
    mouse::Cursor,
    renderer::{self, Quad},
    text::Paragraph as _,
    widget::text::{Alignment, LineHeight, Shaping, Wrapping},
    widget::{self, tree::Tree, Widget},
    {keyboard, Border, Element, Event, Length, Point, Rectangle, Size}, {mouse, Text},
};
#[cfg(feature = "iced")]
use iced_renderer::graphics::text::Paragraph;

#[cfg(feature = "libcosmic")]
use cosmic::iced_core::{
    self,
    layout::{self, Layout},
    mouse::Cursor,
    renderer::{self, Quad},
    text::Paragraph as _,
    widget::text::{LineHeight, Shaping, Wrapping},
    widget::{self, tree::Tree, Widget},
    {keyboard, Border, Event, Length, Point, Rectangle, Size}, {mouse, Text},
};
#[cfg(feature = "libcosmic")]
use cosmic::iced_widget::graphics::text::Paragraph;

#[cfg(feature = "iced")]
pub struct MemoryEditor<'a, Context, Theme>
where
    Theme: Catalog + iced_core::widget::text::Catalog,
    Context: context::MemoryEditorContext,
{
    content: &'a Content<Context>,
    class: <Theme as crate::style::Catalog>::Class<'a>,
    style: Option<Style>,
}

#[cfg(feature = "iced")]
impl<'a, Context, Theme> MemoryEditor<'a, Context, Theme>
where
    Theme: Catalog + iced_core::widget::text::Catalog + 'a,
    Context: context::MemoryEditorContext,
{
    pub fn new(content: &'a Content<Context>) -> Self {
        MemoryEditor {
            class: <Theme as crate::style::Catalog>::default(),
            content,
            style: None,
        }
    }
}

#[cfg(feature = "libcosmic")]
pub struct MemoryEditor<'a, Context>
where
    Context: context::MemoryEditorContext + 'a,
{
    content: &'a Content<Context>,
    style: Option<Style>,
}

#[cfg(feature = "libcosmic")]
impl<'a, Context> MemoryEditor<'a, Context>
where
    Context: context::MemoryEditorContext + 'a,
{
    pub fn new(content: &'a Content<Context>) -> Self {
        MemoryEditor { content, style: None }
    }
}

#[cfg(feature = "iced")]
impl<'a, Context, Theme> MemoryEditor<'a, Context, Theme>
where
    Theme: Catalog + iced_core::widget::text::Catalog + 'a,
    Context: context::MemoryEditorContext + 'a,
{
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }
}

#[cfg(feature = "libcosmic")]
impl<'a, Context> MemoryEditor<'a, Context>
where
    Context: context::MemoryEditorContext + 'a,
{
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }
}

#[cfg(feature = "iced")]
impl<'a, Context, Theme, Message, Renderer> Widget<Message, Theme, Renderer>
    for MemoryEditor<'a, Context, Theme>
where
    Renderer: renderer::Renderer + iced_core::text::Renderer<Font = iced_core::Font> + 'a,
    Theme: Catalog + iced_core::widget::text::Catalog + 'a,
    Context: context::MemoryEditorContext + 'a,
{
    fn size(&self) -> Size<Length> {
        size()
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(self.content, tree, renderer, limits)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: iced_core::mouse::Cursor,
        _viewport: &Rectangle,
    ) where
        Theme: Catalog,
    {
        let style = self.style.unwrap_or_else(|| <Theme as crate::style::Catalog>::style(theme, &self.class));
        draw(self.content, tree, renderer, &style, layout);
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced_core::Clipboard,
        shell: &mut iced_core::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        if iced_core::event::Status::Captured == update(self.content, tree, event.clone(), layout, cursor, shell) {
            shell.request_redraw();
        }
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn diff(&self, _tree: &mut Tree) {}

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn children(&self) -> Vec<Tree> {
        Vec::new()
    }
}

#[cfg(feature = "libcosmic")]
impl<'a, Context, Message, Renderer> Widget<Message, cosmic::Theme, Renderer>
    for MemoryEditor<'a, Context>
where
    Context: context::MemoryEditorContext + 'a,
    Renderer: iced_core::Renderer + iced_core::text::Renderer<Font = iced_core::Font> + 'a,
{
    fn size(&self) -> Size<Length> {
        size()
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(self.content, tree, renderer, limits)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &cosmic::Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: iced_core::mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        draw(self.content, tree, renderer, &self.style.unwrap_or_else(|| theme.into()), layout);
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced_core::Clipboard,
        shell: &mut iced_core::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> iced_core::event::Status {
        update(self.content, tree, event, layout, cursor, shell)
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn diff(&mut self, _tree: &mut Tree) {}

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }
}

fn size() -> Size<Length> {
    Size {
        width: Length::Shrink,
        height: Length::Fill,
    }
}

fn layout<'a, Renderer, Context>(
    content: &Content<Context>,
    tree: &mut widget::Tree,
    renderer: &Renderer,
    limits: &layout::Limits,
) -> layout::Node
where
    Renderer: iced_core::Renderer + iced_core::text::Renderer<Font = iced_core::Font> + 'a,
    Context: context::MemoryEditorContext + 'a,
{
    let state = tree.state.downcast_mut::<State>();
    let text_size = renderer.default_size();
    let text_line_height = LineHeight::default();
    let options = options(content);

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
        font: iced_core::Font::MONOSPACE,
        #[cfg(feature = "libcosmic")]
        horizontal_alignment: iced_core::alignment::Horizontal::Left,
        #[cfg(feature = "iced")]
        align_x: Alignment::Left,
        #[cfg(feature = "libcosmic")]
        vertical_alignment: iced_core::alignment::Vertical::Top,
        #[cfg(feature = "iced")]
        align_y: Vertical::Top,
        shaping: Shaping::Advanced,
        wrapping: Wrapping::None,
    })
    .min_bounds()
    .width;

    if cfg!(feature = "libcosmic") {  // TODO: Explain/remove
        state.dimensions.char_width *= 1.2;
    }

    state.update_dimensions(options.row_length as f32);

    layout::Node::with_children(limits.max(), vec![])
}

fn draw<'a, Renderer, Context>(
    content: &Content<Context>,
    tree: &widget::Tree,
    renderer: &mut Renderer,
    style: &Style,
    layout: Layout<'_>,
) where
    Renderer: iced_core::Renderer + iced_core::text::Renderer<Font = iced_core::Font> + 'a,
    Context: context::MemoryEditorContext + 'a,
{
    let state = tree.state.downcast_ref::<State>();
    let bounds = layout.bounds();
    let options = options(content);

    background(renderer, &style, bounds);

    let separator_bounds = Rectangle {
        x: bounds.x,
        y: bounds.y,
        height: state.dimensions.row_count as f32 * state.dimensions.char_height,
        width: bounds.width,
    };

    separator(
        renderer,
        &style,
        separator_bounds,
        state.dimensions.address_separator_x,
    );

    if options.show_ascii {
        separator(
            renderer,
            &style,
            separator_bounds,
            state.dimensions.ascii_separator_x,
        );
    }

    let mut addr = state.start_address;
    let mut y_offset = bounds.y;

    for slice in state
        .data
        .chunks_exact(options.row_length)
        .take(state.dimensions.row_count)
    {
        row(
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

    bottom_panel(content, tree, renderer, state, &style, bounds, &options);
    options_panel(tree, renderer, &style, layout, &options);
}

fn update<'a, Context, Message>(
    content: &Content<Context>,
    tree: &mut Tree,
    event: Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    shell: &mut iced_core::Shell<'_, Message>,
) -> iced_core::event::Status
where
    Context: context::MemoryEditorContext + 'a,
{
    let state = tree.state.downcast_mut::<State>();
    let bounds = layout.bounds();
    let options = options(content);

    state.update_bounds(&bounds);

    if state.data.is_empty() {
        update_data(content, state, options.row_length);
        return iced_core::event::Status::Captured;
    }

    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
            if let (true, message) = handle_mouse_interaction(content, state, cursor, bounds, &options)
            {
                if let Some(message) = message {
                    match message {
                        Action::OptionsToggled => {
                            state.options_open = !state.options_open;
                            shell.invalidate_layout();
                        }
                        Action::RowLengthUpdate(len) => {
                            write_options(content, MemoryEditorOptions {
                                row_length: len,
                                ..options
                            });
                            update_data(content, state, len);
                            shell.invalidate_layout();
                        }
                        Action::PreviewFormatUpdate(format) => {
                            write_options(content,MemoryEditorOptions {
                                preview_data_format: format,
                                ..options
                            });
                        }
                        Action::ShowASCIIUpdate(show) => {
                            write_options(content, MemoryEditorOptions {
                                show_ascii: show,
                                ..options
                            });
                            shell.invalidate_layout();
                        }
                        _ => (),
                    };
                }
                return iced_core::event::Status::Captured;
            }
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) if state.addr_input.focused => {
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
                            return iced_core::event::Status::Captured;
                        }
                    }
                }
                keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                    state.addr_input.value.pop();
                    return iced_core::event::Status::Captured;
                }
                keyboard::Key::Named(keyboard::key::Named::Enter) => {
                    if let Ok(addr) = usize::from_str_radix(&state.addr_input.value, 16) {
                        state.start_address = addr;
                        update_data(content, state, options.row_length);
                        state.addr_input.value.clear();
                    }
                    state.addr_input.focused = false;
                    return iced_core::event::Status::Captured;
                }
                keyboard::Key::Named(keyboard::key::Named::Escape) => {
                    state.addr_input.focused = false;
                    return iced_core::event::Status::Captured;
                }
                _ => {}
            }
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) if state.byte_input.focused => {
            match key {
                keyboard::Key::Character(c) => {
                    if state.byte_input.value.len() < 2
                        && c.as_str()
                            .chars()
                            .next()
                            .is_some_and(|ch| ch.is_ascii_hexdigit())
                    {
                        if let Some(ch) = c.as_str().chars().next() {
                            state.byte_input.value.push(ch);
                            return iced_core::event::Status::Captured;
                        }
                    }
                }
                keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                    state.byte_input.value.pop();
                    return iced_core::event::Status::Captured;
                }
                keyboard::Key::Named(keyboard::key::Named::Enter) => {
                    if let Some(selected_addr) = state.selected_address {
                        if let Ok(byte) = u8::from_str_radix(&state.byte_input.value, 16) {
                            write(content,selected_addr, byte);
                            update_data(content, state, options.row_length);
                            state.byte_input.value.clear();
                        }
                    }
                    state.byte_input.focused = false;
                    return iced_core::event::Status::Captured;
                }
                keyboard::Key::Named(keyboard::key::Named::Escape) => {
                    state.byte_input.focused = false;
                    return iced_core::event::Status::Captured;
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
            update_data(content, state, options.row_length);
            return iced_core::event::Status::Captured;
        }
        _ => (),
    }
    iced_core::event::Status::Ignored
}

fn options<Context: MemoryEditorContext>(content: &Content<Context>) -> MemoryEditorOptions {
    content.internal.borrow().context.options()
}

fn update_data<Context: MemoryEditorContext>(
    content: &Content<Context>,
    state: &mut State,
    len: usize,
) {
    state.data = content.internal.borrow().context.data(Range::<usize> {
        start: state.start_address,
        end: state.start_address + state.dimensions.row_count * len,
    });
}

fn write<Context: MemoryEditorContext>(content: &Content<Context>, addr: usize, data: u8) {
    content.internal.borrow_mut().context.write(addr, data);
}

fn write_options<Context: MemoryEditorContext>(
    content: &Content<Context>,
    options: MemoryEditorOptions,
) {
    content.internal.borrow_mut().context.write_options(options);
}

fn handle_mouse_interaction<Context: MemoryEditorContext>(
    content: &Content<Context>,
    state: &mut State,
    cursor: Cursor,
    bounds: Rectangle,
    options: &MemoryEditorOptions,
) -> (bool, Option<Action>) {
    if state.addr_input.focused {
        state.addr_input.focused = false;
    }

    if state.byte_input.focused {
        state.byte_input.focused = false;
    }

    if cursor.position().is_none() {
        return (false, None);
    }

    let position = cursor.position().unwrap();

    if cursor.is_over(state.bounds.options) {
        return (true, Some(Action::OptionsToggled));
    }

    if cursor.is_over(state.bounds.addr_input) {
        state.addr_input.focused = true;
        state.addr_input.value.clear();
        return (true, None);
    }

    if let Some(selected_addr) = state.selected_address {
        if cursor.is_over(state.bounds.byte_input)
            && content.internal.borrow().context.can_write(selected_addr)
        {
            state.byte_input.focused = true;
            state.byte_input.value.clear();
            return (true, None);
        }
    }

    if state.options_open {
        if cursor.is_over(state.bounds.show_ascii_checkbox) {
            return (true, Some(Action::ShowASCIIUpdate(!options.show_ascii)));
        }

        if cursor.is_over(state.bounds.prev_format) {
            return (
                true,
                Some(Action::PreviewFormatUpdate(options.previous_data_format())),
            );
        }

        if cursor.is_over(state.bounds.next_format) {
            return (
                true,
                Some(Action::PreviewFormatUpdate(options.next_data_format())),
            );
        }

        if cursor.is_over(state.bounds.prev_row_length) {
            return (
                true,
                Some(Action::RowLengthUpdate((options.row_length - 8).max(8))),
            );
        }

        if cursor.is_over(state.bounds.next_row_length) {
            return (true, Some(Action::RowLengthUpdate(options.row_length + 8)));
        }
    }

    let row_index = ((position.y - bounds.y) / state.dimensions.char_height).trunc() as usize;
    if row_index >= state.dimensions.row_count {
        state.selected_address = None;
        state.text.value_text.clear();
        return (true, None);
    }

    let x_in_data = position.x - (bounds.x + state.dimensions.section_data_start);
    if x_in_data < 0.0 {
        state.selected_address = None;
        state.text.value_text.clear();
        return (true, None);
    }

    let byte_index = calculate_byte_index(x_in_data, state);
    if byte_index >= options.row_length {
        state.selected_address = None;
        state.text.value_text.clear();
        return (true, None);
    }

    let clicked_address = state.start_address + (row_index * options.row_length) + byte_index;
    state.selected_address = Some(clicked_address);
    state.text.value_text = format!("{:06X} =", clicked_address);
    state.text.value_len = state.text.value_text.len() as f32 * state.dimensions.char_width;

    (true, None)
}

fn calculate_byte_index(x_in_data: f32, state: &State) -> usize {
    let total_x = x_in_data + state.dimensions.group_spacing / 2.0;
    let byte_slot_width = state.dimensions.byte_width
        + (state.dimensions.group_spacing / state.dimensions.group_char_len as f32);

    (total_x / byte_slot_width) as usize
}

fn format_preview_value(data: &[u8], format: &PreviewDataFormat) -> String {
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

fn background<Renderer>(renderer: &mut Renderer, style: &Style, bounds: Rectangle)
where
    Renderer: renderer::Renderer,
{
    renderer.fill_quad(
        renderer::Quad {
            bounds,
            ..Default::default()
        },
        style.background,
    );
}

fn separator<Renderer>(renderer: &mut Renderer, style: &Style, bounds: Rectangle, x: f32)
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
            ..Default::default()
        },
        style.text_color,
    );
}

fn row<Renderer>(
    renderer: &mut Renderer,
    style: &Style,
    bounds: Rectangle,
    state: &State,
    addr: &usize,
    row_data: &[u8],
    options: &MemoryEditorOptions,
) where
    Renderer: renderer::Renderer + iced_core::text::Renderer<Font = iced_core::Font>,
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
                    ..Default::default()
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
                        ..Default::default()
                    },
                    style.selection_color,
                );
            }
        }
    }
}

fn options_panel<Renderer>(
    tree: &widget::Tree,
    renderer: &mut Renderer,
    style: &Style,
    layout: Layout<'_>,
    options: &MemoryEditorOptions,
) where
    Renderer: renderer::Renderer + iced_core::text::Renderer<Font = iced_core::Font>,
{
    let state = tree.state.downcast_ref::<State>();

    if !state.options_open {
        return;
    }

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
            ..Default::default()
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
                ..Default::default()
            },
            style.text_color,
        );
    }
}

fn bottom_panel<Renderer, Context: MemoryEditorContext>(
    content: &Content<Context>,
    _tree: &widget::Tree,
    renderer: &mut Renderer,
    state: &State,
    style: &Style,
    bounds: Rectangle,
    options: &MemoryEditorOptions,
) where
    Renderer: renderer::Renderer + iced_core::text::Renderer<Font = iced_core::Font>,
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
                ..Default::default()
            },
            ..Default::default()
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
            ..Default::default()
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
        if content.internal.borrow().context.can_write(selected_addr) {
            let value_bounds = Rectangle {
                x: state.bounds.byte_input.x - state.text.value_len - state.dimensions.char_width,
                y: panel_bounds.y,
                width: state.text.value_len,
                height: panel_bounds.height,
            };

            renderer.fill_text(
                Text {
                    content: state.text.value_text.clone(),
                    bounds: Size::new(state.text.value_len, panel_bounds.height),
                    ..state.text_defaults
                },
                Point::new(
                    value_bounds.x,
                    panel_bounds.y + state.dimensions.char_width / 2.0,
                ),
                style.text_color,
                value_bounds,
            );

            let byte_input_bounds = state.bounds.byte_input;

            renderer.fill_quad(
                Quad {
                    bounds: byte_input_bounds,
                    border: Border {
                        width: 1.0,
                        color: if state.byte_input.focused {
                            style.text_color
                        } else {
                            style.border.color
                        },
                        ..style.border
                    },
                    ..Default::default()
                },
                style.primary_color,
            );

            renderer.fill_text(
                Text {
                    content: state.byte_input.value.clone(),
                    bounds: Size::new(
                        byte_input_bounds.width - state.dimensions.char_width,
                        byte_input_bounds.height,
                    ),
                    ..state.text_defaults
                },
                Point::new(
                    byte_input_bounds.x + state.dimensions.char_width / 2.0,
                    panel_bounds.y + state.dimensions.char_width / 2.0,
                ),
                style.text_color,
                byte_input_bounds,
            );
        }
    }

    if let Some(selected_addr) = state.selected_address {
        if selected_addr >= state.start_address
            && selected_addr < state.start_address + state.dimensions.row_count * options.row_length
        {
            let required_bytes = match options.preview_data_format {
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

            let value_text = format_preview_value(
                &preview_data[..required_bytes],
                &options.preview_data_format,
            );

            let value_width = value_text.len() as f32 * state.dimensions.char_width;
            let value_bound = Rectangle {
                x: panel_bounds.x + panel_bounds.width - value_width - state.dimensions.char_width,
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

#[cfg(feature = "iced")]
impl<'a, Context, Theme, Message, Renderer> From<MemoryEditor<'a, Context, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced_core::text::Renderer<Font = iced_core::Font> + 'a,
    Theme: Catalog + iced_core::widget::text::Catalog + 'a,
    Context: context::MemoryEditorContext + 'a,
{
    fn from(memory_editor: MemoryEditor<'a, Context, Theme>) -> Self {
        Self::new(memory_editor)
    }
}

#[cfg(feature = "iced")]
pub fn memory_editor<'a, Context, Theme>(
    content: &'a Content<Context>,
) -> MemoryEditor<'a, Context, Theme>
where
    Context: context::MemoryEditorContext + 'a,
    Theme: Catalog + iced_core::widget::text::Catalog + 'a,
{
    MemoryEditor::new(content)
}

#[cfg(feature = "libcosmic")]
impl<'a, Context, Message> From<MemoryEditor<'a, Context>> for cosmic::Element<'a, Message>
where
    Context: context::MemoryEditorContext + 'a,
{
    fn from(memory_editor: MemoryEditor<'a, Context>) -> Self {
        Self::new(memory_editor)
    }
}

#[cfg(feature = "libcosmic")]
pub fn memory_editor<'a, Context>(content: &'a Content<Context>) -> MemoryEditor<'a, Context>
where
    Context: context::MemoryEditorContext + 'a,
{
    MemoryEditor::new(content)
}
