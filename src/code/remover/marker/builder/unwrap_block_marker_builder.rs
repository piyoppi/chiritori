use std::{ops::Range, rc::Rc};
use crate::{code::utils::line_break_pos_finder::{find_next_line_break_pos, find_prev_line_break_pos}, parser::Element};
use super::MarkerBuilder;

pub struct UnwrapBlockMarkerBuilder {
    pub content: Rc<String>
}

impl MarkerBuilder for UnwrapBlockMarkerBuilder {
    fn build(&self, el: &Element) -> (Range<usize>, Option<Range<usize>>) {
        let bytes = self.content.as_bytes();
        let start_el_remove_end_pos = find_next_line_break_pos(self.content.as_ref(), bytes, el.start_token.byte_end, false)
            .and_then(|pos| find_next_line_break_pos(self.content.as_ref(), bytes, pos + 1, false));
        let end_el_remove_start_pos = find_prev_line_break_pos(self.content.as_ref(), bytes, el.end_token.byte_start, false)
            .and_then(|pos| find_prev_line_break_pos(self.content.as_ref(), bytes, pos, false));

        match (start_el_remove_end_pos, end_el_remove_start_pos) {
            (Some(end), Some(start)) => (
                el.start_token.byte_start..end,
                Some(start + 1..el.end_token.byte_end)
            ),
            _ => (el.start_token.byte_start..el.end_token.byte_end, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{parser, tokenizer};
    use super::*;

    #[test]
    fn test_build() {
        //                              10        20        30
        //                     012345678901234567890123456789012345
        //                     |       ^---------^  ^----------^                  
        let content = Rc::new("foo+bar+<remove>+{+b+}+</remove>+baz".replace('+', "\n"));

        let builder = UnwrapBlockMarkerBuilder {
            content: Rc::clone(&content)
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
                _ => None
            })
            .unwrap();

        assert_eq!(builder.build(&parsed), (8..18, Some(21..32)));
    }
}
