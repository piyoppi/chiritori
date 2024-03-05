use crate::element_parser::Element;
use crate::remover::RemoveMarker;
use crate::tokenizer::Token;

pub trait RemoveMarkerBuilder {
    fn create_remove_marker(
        &self,
        start_token: &Token,
        start_el: &Element,
        end_token: &Token,
    ) -> Option<RemoveMarker>;
}
