use std::fmt::Display;

#[derive(Clone, Debug, Default, Copy, PartialEq)]
pub enum PreviewDataFormat {
    #[default]
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}

impl Display for PreviewDataFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PreviewDataFormat::U8 => "U8",
                PreviewDataFormat::U16 => "U16",
                PreviewDataFormat::U32 => "U32",
                PreviewDataFormat::U64 => "U64",
                PreviewDataFormat::I8 => "I8",
                PreviewDataFormat::I16 => "I16",
                PreviewDataFormat::I32 => "I32",
                PreviewDataFormat::I64 => "I64",
                PreviewDataFormat::F32 => "F32",
                PreviewDataFormat::F64 => "F64",
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryEditorOptions {
    pub row_length: usize,
    pub preview_data_format: PreviewDataFormat,
    pub show_ascii: bool,
}

impl Default for MemoryEditorOptions {
    fn default() -> Self {
        Self {
            row_length: 16,
            preview_data_format: PreviewDataFormat::default(),
            show_ascii: true,
        }
    }
}

impl MemoryEditorOptions {
    pub fn previous_data_format(&self) -> PreviewDataFormat {
        match self.preview_data_format {
            PreviewDataFormat::U8 => PreviewDataFormat::F64,
            PreviewDataFormat::U16 => PreviewDataFormat::U8,
            PreviewDataFormat::U32 => PreviewDataFormat::U16,
            PreviewDataFormat::U64 => PreviewDataFormat::U32,
            PreviewDataFormat::I8 => PreviewDataFormat::U64,
            PreviewDataFormat::I16 => PreviewDataFormat::I8,
            PreviewDataFormat::I32 => PreviewDataFormat::I16,
            PreviewDataFormat::I64 => PreviewDataFormat::I32,
            PreviewDataFormat::F32 => PreviewDataFormat::I64,
            PreviewDataFormat::F64 => PreviewDataFormat::F32,
        }
    }

    pub fn next_data_format(&self) -> PreviewDataFormat {
        match self.preview_data_format {
            PreviewDataFormat::U8 => PreviewDataFormat::U16,
            PreviewDataFormat::U16 => PreviewDataFormat::U32,
            PreviewDataFormat::U32 => PreviewDataFormat::U64,
            PreviewDataFormat::U64 => PreviewDataFormat::I8,
            PreviewDataFormat::I8 => PreviewDataFormat::I16,
            PreviewDataFormat::I16 => PreviewDataFormat::I32,
            PreviewDataFormat::I32 => PreviewDataFormat::I64,
            PreviewDataFormat::I64 => PreviewDataFormat::F32,
            PreviewDataFormat::F32 => PreviewDataFormat::F64,
            PreviewDataFormat::F64 => PreviewDataFormat::U8,
        }
    }
}

