use super::MarkerAvailability;

#[derive(Default)]
pub struct OpenStructureMarkerAvailability {}

impl MarkerAvailability for OpenStructureMarkerAvailability {
    fn is_available(&self, element: &crate::parser::Element) -> bool {
        element.start_element.attrs.iter().any(|a| a.name == "open-structure")
    }
}
