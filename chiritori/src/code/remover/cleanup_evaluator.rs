pub mod time_limited_evaluator;

use crate::element_parser::Element;

pub trait CleanupEvaluator {
    fn is_cleanup(&self, start_el: &Element) -> bool;
}
