use super::MarkerAvailability;

pub struct UnwrapBlockMarkerAvailability {
    tag_name: &'static str
}

impl UnwrapBlockMarkerAvailability {
    pub fn new(tag_name: &'static str) -> Self {
        Self {
            tag_name
        }
    }
}

impl MarkerAvailability for UnwrapBlockMarkerAvailability {
    fn is_available(&self, element: &crate::parser::Element) -> bool {
        element.start_element.attrs.iter().any(|a| a.name == self.tag_name)
    }
}
