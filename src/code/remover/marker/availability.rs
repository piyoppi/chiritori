pub mod block_marker_availability;
pub mod open_structure_marker_availability;

use crate::parser::Element;

pub trait MarkerAvailability {
    fn is_available(&self, element: &Element) -> bool;
}
