use crate::options::{MemoryEditorOptions, PreviewDataFormat};
use std::{cell::RefCell, ops::Range};

pub trait MemoryEditorContext {
    fn data(&self, range: Range<usize>) -> Vec<u8>;
    fn options(&self) -> MemoryEditorOptions;
    fn write_options(&mut self, options: MemoryEditorOptions);
    fn can_write(&self, address: usize) -> bool;
    fn write(&mut self, address: usize, value: u8);
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    ShowASCIIUpdate(bool),
    PreviewFormatUpdate(PreviewDataFormat),
    RowLengthUpdate(usize),
    UpdateByte(usize, u8),
    OptionsToggled,
}

pub struct Content<C: MemoryEditorContext> {
    pub internal: RefCell<Internal<C>>
}

pub struct Internal<C: MemoryEditorContext>
{
    pub context: C,
}

impl<C: MemoryEditorContext> Content<C>
{
    pub fn new(context: C) -> Self {
        Self {
            internal: RefCell::new(Internal {
                context,
            })
        }
    }
}