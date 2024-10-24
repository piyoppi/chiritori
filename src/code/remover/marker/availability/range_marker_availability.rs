use super::MarkerAvailability;

#[derive(Default)]
pub struct RangeMarkerAvailability {}

impl MarkerAvailability for RangeMarkerAvailability {
    fn is_available(&self, _element: &crate::parser::Element) -> bool {
        true
    }
}
