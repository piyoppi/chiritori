use crate::{code::remover::marker::{Range, RemoveMarker}, parser::Element};

use super::MarkerBuilder;

#[derive(Default)]
pub struct BlockMarkerBuilder {}

impl MarkerBuilder for BlockMarkerBuilder {
    fn build(&self, el: &Element) -> RemoveMarker {
        RemoveMarker::Block(Range {
            byte_start: el.start_token.byte_start,
            byte_end: el.end_token.byte_end,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use crate::{parser, tokenizer};
    use super::*;

    #[test]
    fn test_build() {
        //                              10        20        30
        //                     012345678901234567890123456789012345
        //                     |       ^-----------------------^                  
        let content = Rc::new("foo+bar+<remove>+a+b+c+</remove>+baz".replace('+', "\n"));

        let builder = BlockMarkerBuilder::default();

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

        assert_eq!(
            builder.build(&parsed),
            RemoveMarker::Block(
                Range {
                    byte_start: 8,
                    byte_end: 32,
                },
            )
        );
    }
}
