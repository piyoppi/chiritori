use crate::element_parser::Element;
use crate::remover::remove_marker_builder::RemoveMarkerBuilder;
use crate::remover::RemoveMarker;
use crate::tokenizer::Token;
use chrono::{DateTime, Local};

#[derive(Debug, PartialEq, Clone)]
pub struct TimeLimitedRemover {
    pub current_time: DateTime<Local>,
    pub time_offset: String,
}

impl RemoveMarkerBuilder for TimeLimitedRemover {
    fn create_remove_marker(
        &self,
        start_token: &Token,
        start_el: &Element,
        end_token: &Token,
    ) -> Option<RemoveMarker> {
        let expires_attr = start_el.attrs.iter().find(|a| a.name == "to");

        if expires_attr.is_none() || expires_attr.unwrap().value.is_none() {
            return None;
        }

        let mut expires_str = expires_attr.unwrap().value.clone().unwrap().to_string();
        expires_str.push_str(" ");
        expires_str.push_str(self.time_offset.as_str());
        let expires = DateTime::parse_from_str(&expires_str, "%Y-%m-%d %H:%M:%S %z");

        if expires.is_err() {
            return None;
        }

        if self.current_time < expires.unwrap() {
            return None;
        }

        Some(RemoveMarker {
            byte_start: start_token.byte_start,
            byte_end: end_token.byte_end,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element_parser::Attribute;
    use crate::tokenizer::{ElementToken, TokenKind};

    #[test]
    fn test_remove_marker() {
        let start_token = Token {
            kind: TokenKind::Element(ElementToken {
                delimiter_start: "<!--",
                delimiter_end: "-->",
            }),
            value: "<!-- time-limited to='2021-12-31 23:00:00' -->",
            start: 9,
            byte_start: 9,
            end: 43,
            byte_end: 44,
        };
        let start_el = Element {
            name: "time-limited",
            attrs: vec![Attribute {
                name: "to",
                value: Some("2021-12-31 23:00:00"),
            }],
        };
        let end_token = Token {
            kind: TokenKind::Element(ElementToken {
                delimiter_start: "<!--",
                delimiter_end: "-->",
            }),
            value: "<!-- /time-limited -->",
            start: 45,
            byte_start: 45,
            end: 67,
            byte_end: 68,
        };

        let remover = TimeLimitedRemover {
            current_time: DateTime::parse_from_str(
                "2022-01-01 00:00:00 +0000",
                "%Y-%m-%d %H:%M:%S %z",
            )
            .unwrap()
            .into(),
            time_offset: "+0000".to_string(),
        };
        assert_eq!(
            remover.create_remove_marker(&start_token, &start_el, &end_token),
            Some(RemoveMarker {
                byte_start: 9,
                byte_end: 68,
            })
        );
    }
}
