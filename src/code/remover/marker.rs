pub mod builder;
pub mod availability;
pub mod factory;

#[derive(Debug, PartialEq)]
pub struct Range {
    pub byte_start: usize,
    pub byte_end: usize,
}

#[derive(Debug, PartialEq)]
pub enum RemoveMarker {
    Block(Range),
    OpenStructure(Range, Range)
}
