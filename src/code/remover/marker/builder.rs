pub mod range_marker_builder;
pub mod unwrap_block_marker_builder;

use std::ops::Range;
use crate::parser::Element;

pub trait MarkerBuilder {
    fn build(&self, element: &Element) -> (Range<usize>, Option<Range<usize>>);
}
