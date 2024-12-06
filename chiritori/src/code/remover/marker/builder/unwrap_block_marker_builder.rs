use super::MarkerBuilder;
use crate::{
    code::utils::line_break_pos_finder::{find_next_line_break_pos, find_prev_line_break_pos},
    parser::Element,
};
use std::{ops::Range, rc::Rc};

pub struct UnwrapBlockMarkerBuilder {
    pub content: Rc<String>,
}

impl MarkerBuilder for UnwrapBlockMarkerBuilder {
    fn build(&self, el: &Element) -> (Range<usize>, Option<Range<usize>>) {
        let bytes = self.content.as_bytes();
        let start_el_remove_end_pos =
            find_next_line_break_pos(self.content.as_ref(), bytes, el.start_token.byte_end, false)
                .and_then(|pos| {
                    find_next_line_break_pos(self.content.as_ref(), bytes, pos + 1, false)
                });
        let end_el_remove_start_pos =
            find_prev_line_break_pos(self.content.as_ref(), bytes, el.end_token.byte_start, false)
                .and_then(|pos| find_prev_line_break_pos(self.content.as_ref(), bytes, pos, false));

        // If the range is invalid, do nothing.
        match (start_el_remove_end_pos, end_el_remove_start_pos) {
            (Some(end), Some(start)) => {
                if start > end {
                    (
                        el.start_token.byte_start..end,
                        Some(start + 1..el.end_token.byte_end),
                    )
                } else {
                    (el.start_token.byte_start..el.start_token.byte_start, None)
                }
            }
            _ => (el.start_token.byte_start..el.start_token.byte_start, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::{parser, tokenizer};

    #[rstest]
    //             10        20        30
    //     012345678901234567890123456789012345
    //     |       ^---------^  ^----------^
    #[case("foo+bar+<remove>+{+b+}+</remove>+baz", 8..18, Some(21..32))]
    #[case("foo+bar+<remove> {b} </remove>+baz+", 8..8, None)]
    fn test_build(
        #[case] input: String,
        #[case] expected_start_range: Range<usize>,
        #[case] expected_end_range: Option<Range<usize>>,
    ) {
        let content = Rc::new(input.replace('+', "\n"));

        let builder = UnwrapBlockMarkerBuilder {
            content: Rc::clone(&content),
        };

        let tokens = tokenizer::tokenize(&content, "<", ">");
        let parsed = parser::parse(&tokens)
            .into_iter()
            .find_map(|c| match c {
                parser::ContentPart::Element(el) => {
                    if el.start_element.name == "remove" {
                        Some(el)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .unwrap();

        assert_eq!(
            builder.build(&parsed),
            (expected_start_range, expected_end_range)
        );
    }
}
