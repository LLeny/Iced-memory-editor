use crate::options::{MemoryEditorOptions, PreviewDataFormat};
use std::ops::Range;

pub trait MemoryEditorContext: Default + Sized {
    fn perform(&mut self, action: Action);
    fn data(&self) -> Vec<u8>;
    fn options(&self) -> MemoryEditorOptions;
    fn is_empty(&self) -> bool;
    fn can_write(&self, address: usize) -> bool;
    fn write(&mut self, address: usize, value: u8);
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    DataUpdate(Range<usize>),
    ShowASCIIUpdate(bool),
    PreviewFormatUpdate(PreviewDataFormat),
    RowLengthUpdate(usize),
    UpdateByte(usize, u8),
}
