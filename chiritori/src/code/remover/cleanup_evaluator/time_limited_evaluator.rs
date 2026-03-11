use super::CleanupEvaluator;
use crate::element_parser::Element;
use chrono::{DateTime, Local};

#[derive(Debug, PartialEq, Clone)]
pub struct TimeLimitedEvaluator {
    pub current_time: DateTime<Local>,
    pub time_offset: String,
}

impl CleanupEvaluator for TimeLimitedEvaluator {
    fn is_cleanup(&self, start_el: &Element) -> bool {
        let to = start_el.attrs.iter().find(|a| a.name == "to");

        if to.is_some() {
            return false;
        }

        let from = start_el.attrs.iter().find(|a| a.name == "from");

        if from.is_none() || from.unwrap().value.is_none() {
            return false;
        }

        let mut expires_str = from.unwrap().value.unwrap().to_string();
        expires_str.push(' ');
        expires_str.push_str(self.time_offset.as_str());
        let expires = DateTime::parse_from_str(&expires_str, "%Y-%m-%d %H:%M:%S %z");

        if expires.is_err() {
            return false;
        }

        if self.current_time < expires.unwrap() {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element_parser::Attribute;

    #[test]
    fn test_cleanup_marker_outdated() {
        let start_el = Element {
            name: "time-limited",
            attrs: vec![Attribute {
                name: "from",
                value: Some("2021-12-31 23:00:00"),
            }],
        };
        let evaluator = TimeLimitedEvaluator {
            current_time: DateTime::parse_from_str(
                "2022-01-01 00:00:00 +0000",
                "%Y-%m-%d %H:%M:%S %z",
            )
            .unwrap()
            .into(),
            time_offset: "+0000".to_string(),
        };
        assert!(evaluator.is_cleanup(&start_el));
    }

    #[test]
    fn test_remove_marker_in_term() {
        let start_el = Element {
            name: "time-limited",
            attrs: vec![Attribute {
                name: "to",
                value: Some("2023-12-31 23:59:59"),
            }],
        };
        let evaluator = TimeLimitedEvaluator {
            current_time: DateTime::parse_from_str(
                "2022-01-01 00:00:00 +0000",
                "%Y-%m-%d %H:%M:%S %z",
            )
            .unwrap()
            .into(),
            time_offset: "+0000".to_string(),
        };
        assert!(!evaluator.is_cleanup(&start_el));
    }
}
