use super::remover::RemovableRange;
use crate::parser::Element;

pub fn clean(el: &Element) -> RemovableRange {
    (
        el.start_token.byte_start..el.start_token.byte_end,
        Some(el.end_token.byte_start..el.end_token.byte_end),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parser, tokenizer};
    use std::rc::Rc;

    #[test]
    fn test_clean() {
        //                              10        20        30
        //                     012345678901234567890123456789012345
        //                     |       ^------^       ^--------^
        let content = Rc::new("foo+bar+<remove>+a+b+c+</remove>+baz".replace('+', "\n"));

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

        let built = clean(&parsed);
        assert_eq!(built, (8..16, Some(23..32)));
    }
}
