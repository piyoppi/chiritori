use crate::element_parser::Element;

pub trait RemovalEvaluator {
    fn is_removal(&self, start_el: &Element) -> bool;
}
