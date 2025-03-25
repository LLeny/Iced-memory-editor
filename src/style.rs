#[cfg(feature = "iced")]
use iced_core::{Border, Color, Shadow, Theme};
#[cfg(feature = "libcosmic")]
use cosmic::iced_core::{Border, Color, Shadow, Theme};

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub background: Color,
    pub primary_color: Color,
    pub text_color: Color,
    pub inactive_text_color: Color,
    pub border: Border,
    pub shadow: Shadow,
    pub selection_color: Color,
    pub selected_text_color: Color,
}

pub trait Catalog {
    type Class<'a>;
    fn default<'a>() -> Self::Class<'a>;
    fn style(&self, item: &Self::Class<'_>) -> Style;
}

pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn default(theme: &Theme) -> Style {
    let pal = theme.palette();
    let ext = theme.extended_palette();

    Style {
        background: pal.background,
        primary_color: ext.secondary.weak.text,
        text_color: pal.text,
        inactive_text_color: ext.secondary.weak.color,
        selection_color: ext.primary.base.color,
        selected_text_color: ext.primary.base.text,
        border: Border::default(),
        shadow: Shadow::default(),
    }
}
#[cfg(feature = "libcosmic")]
impl From<&cosmic::Theme> for Style {
    fn from(theme: &cosmic::Theme) -> Self {
        let cosmic = theme.cosmic();
        Style {
            background: cosmic.bg_color().into(),
            primary_color: cosmic.primary_container_color().into(),
            text_color: cosmic.button_color().into(),
            inactive_text_color: cosmic.button_bg_color().into(),
            selection_color: cosmic.accent_color().into(),
            selected_text_color: cosmic.primary_container_color().into(),
            border: Border::default(),
            shadow: Shadow::default(),
        }
    }
}
