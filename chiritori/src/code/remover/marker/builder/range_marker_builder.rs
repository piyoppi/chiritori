use super::MarkerBuilder;
use crate::parser::Element;
use std::ops::Range;

#[derive(Default)]
pub struct RangeMarkerBuilder {}

impl MarkerBuilder for RangeMarkerBuilder {
    fn build(&self, el: &Element) -> (Range<usize>, Option<Range<usize>>) {
        (el.start_token.byte_start..el.end_token.byte_end, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parser, tokenizer};
    use std::rc::Rc;

    #[test]
    fn test_build() {
        //                              10        20        30
        //                     012345678901234567890123456789012345
        //                     |       ^-----------------------^
        let content = Rc::new("foo+bar+<remove>+a+b+c+</remove>+baz".replace('+', "\n"));

        let builder = RangeMarkerBuilder::default();

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

        let built = builder.build(&parsed);
        assert_eq!(built, (8..32, None));
    }
}
