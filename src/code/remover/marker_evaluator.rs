use std::collections::HashSet;

use super::removal_evaluator::RemovalEvaluator;
use crate::element_parser::Element;

#[derive(Debug, PartialEq, Clone)]
pub struct MarkerEvaluator {
    pub marker_removal_names: HashSet<String>,
}

impl RemovalEvaluator for MarkerEvaluator {
    fn is_removal(&self, start_el: &Element) -> bool {
        let name_attr_value = start_el
            .attrs
            .iter()
            .find(|a| a.name == "name")
            .and_then(|attr| attr.value);

        if let Some(name_attr_value) = name_attr_value {
            self.marker_removal_names.contains(name_attr_value)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element_parser::Attribute;

    #[test]
    fn test_remove_marker_outdated() {
        let feature1_el = Element {
            name: "marker",
            attrs: vec![Attribute {
                name: "name",
                value: Some("feature1"),
            }],
        };
        let feature2_el = Element {
            name: "marker",
            attrs: vec![Attribute {
                name: "name",
                value: Some("feature2"),
            }],
        };
        let evaluator = MarkerEvaluator {
            marker_removal_names: HashSet::from([String::from("feature1")]),
        };
        assert!(evaluator.is_removal(&feature1_el));
        assert!(!evaluator.is_removal(&feature2_el));
    }
}
