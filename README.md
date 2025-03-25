# Iced Memory Editor

A memory editor widget for the [Iced](https://github.com/iced-rs/iced) / [libcosmic](https://github.com/pop-os/libcosmic) GUI frameworks in Rust.

<p align="center">
    <img src="resources/editor.gif" alt="Editor Preview">
</p>

## Running the Example

Clone the repository and run the example:

```bash
git clone https://github.com/LLeny/iced_memory_editor.git
cd iced_memory_editor
cargo run --release --example memory_editor_iced --features iced
cargo run --release --example memory_editor_cosmic --features libcosmic
```

## Usage in Your Project

Add to your `Cargo.toml`:

#### Iced

```toml
[dependencies]
iced_memory_editor = { git = "https://github.com/LLeny/iced_memory_editor.git", features = ["iced"] }
```

#### Libcosmic

```toml
[dependencies]
iced_memory_editor = { git = "https://github.com/LLeny/iced_memory_editor.git", features = ["libcosmic"] }
```

## Styling
```rust
let memory_editor =
    memory_editor(&self.content).with_style(iced_memory_editor::style::Style {
        background: Color::from_rgb(0.0, 0.0, 0.0),
        primary_color: Color::from_rgb(0.0, 100.0, 0.0),
        text_color: Color::from_rgb(200.0, 200.0, 200.0),
        inactive_text_color: Color::from_rgb(100.0, 100.0, 100.0),
        selection_color: Color::from_rgb(150.0, 0.0, 0.0),
        selected_text_color: Color::from_rgb(200.0, 200.0, 0.0),
        border: Border::default(),
        shadow: Shadow::default(),
    });
```

## License

This project is licensed under the GPLv3 License - see the LICENSE file for details.