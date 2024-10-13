use super::MarkerAvailability;

#[derive(Default)]
pub struct BlockMarkerAvailability {}

impl MarkerAvailability for BlockMarkerAvailability {
    fn is_available(&self, _element: &crate::parser::Element) -> bool {
        true        
    }
}
