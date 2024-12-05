pub mod range_marker_availability;
pub mod unwrap_block_marker_availability;

use crate::parser::Element;

pub trait MarkerAvailability {
    fn is_available(&self, element: &Element) -> bool;
}
