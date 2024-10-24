pub mod range_marker_builder;
pub mod unwrap_block_marker_builder;

use crate::parser::Element;
use std::ops::Range;

pub trait MarkerBuilder {
    fn build(&self, element: &Element) -> (Range<usize>, Option<Range<usize>>);
}
