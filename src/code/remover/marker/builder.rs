pub mod block_marker_builder;
pub mod open_structure_marker_builder;

use crate::parser::Element;
use super::RemoveMarker;

pub trait MarkerBuilder {
    fn build(&self, element: &Element) -> RemoveMarker;
}
