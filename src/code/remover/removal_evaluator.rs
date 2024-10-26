pub mod marker_evaluator;
pub mod time_limited_evaluator;

use crate::element_parser::Element;

pub trait RemovalEvaluator {
    fn is_removal(&self, start_el: &Element) -> bool;
}
