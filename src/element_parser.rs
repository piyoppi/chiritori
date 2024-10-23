use crate::tokenizer;

#[derive(Debug, PartialEq)]
pub struct Element<'a> {
    pub name: &'a str,
    pub attrs: Vec<Attribute<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct Attribute<'a> {
    pub name: &'a str,
    pub value: Option<&'a str>,
}

pub fn parse<'a>(token: &'a tokenizer::Token) -> Option<Element<'a>> {
    #[derive(PartialEq)]
    enum State {
        NameBegin,
        Name(usize),
        NameEnd,
        ValueBegin,
        ValueWithNoQuote,
        ValueWithDoubleQuote(usize),
        ValueWithSingleQuote(usize),
        ParseError,
    }

    match &token.kind {
        tokenizer::TokenKind::Element(element) => {
            let (pairs, last_state) = {
                let target = token
                    .value
                    .trim_start_matches(element.delimiter_start)
                    .trim_end_matches(element.delimiter_end);

                let (mut pairs, last_state) = target
                    .char_indices()
                    .fold((vec![], State::NameBegin), |(mut pairs, mut state), (pos, current_char)| {
                        match state {
                            State::NameBegin => match current_char {
                                ' ' => {}
                                '=' => state = State::ParseError,
                                '"' => state = State::ParseError,
                                '\'' => state = State::ParseError,
                                _ => {
                                    state = State::Name(pos);
                                }
                            },
                            State::Name(start) => match current_char {
                                ' ' => {
                                    pairs.push((&target[start..pos], None));
                                    state = State::NameEnd;
                                }
                                '=' => {
                                    pairs.push((&target[start..pos], None));
                                    state = State::ValueBegin
                                }
                                _ => {}
                            },
                            State::NameEnd => match current_char {
                                ' ' => {}
                                '=' => state = State::ValueBegin,
                                _ => {
                                    state = State::Name(pos);
                                }
                            },
                            State::ValueBegin => match current_char {
                                ' ' => {}
                                '"' => {
                                    state = State::ValueWithDoubleQuote(pos + 1);
                                }
                                '\'' => {
                                    state = State::ValueWithSingleQuote(pos + 1);
                                }
                                _ => state = State::ValueWithNoQuote,
                            },
                            State::ValueWithDoubleQuote(start) => if current_char == '"' {
                                pairs.last_mut().unwrap().1 = Some(&target[start..pos]);
                                state = State::NameBegin
                            },
                            State::ValueWithSingleQuote(start) => if current_char == '\'' {
                                pairs.last_mut().unwrap().1 = Some(&target[start..pos]);
                                state = State::NameBegin
                            },
                            State::ValueWithNoQuote => if current_char == ' ' {
                                state = State::NameBegin
                            },
                            State::ParseError => {}
                        }

                        (pairs, state)
                    });

                if let State::Name(start) = last_state {
                    pairs.push((&target[start..], None));
                }

                (pairs, last_state)
            };

            if last_state == State::ParseError {
                return None;
            }

            let (name, attrs) = (
                pairs[0].0,
                pairs[1..]
                    .iter()
                    .map(|(name, value)| Attribute {
                        name,
                        value: *value
                    })
                .collect()
            );

            Some(Element { name, attrs })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let token = tokenizer::Token {
            kind: tokenizer::TokenKind::Element(tokenizer::ElementToken {
                delimiter_start: "<!--",
                delimiter_end: "-->",
            }),
            value: "<!-- hello-world -->",
            start: 0,
            byte_start: 0,
            end: 17,
            byte_end: 18,
        };

        assert_eq!(
            parse(&token),
            Some(Element {
                name: "hello-world",
                attrs: vec![],
            })
        );

        let token = tokenizer::Token {
            kind: tokenizer::TokenKind::Element(tokenizer::ElementToken {
                delimiter_start: "<!--",
                delimiter_end: "-->",
            }),
            value: "<!-- hello-world from=\"2022-01-01 00:00:00\" to='123' -->",
            start: 0,
            byte_start: 0,
            end: 47,
            byte_end: 48,
        };

        assert_eq!(
            parse(&token),
            Some(Element {
                name: "hello-world",
                attrs: vec![
                    Attribute {
                        name: "from",
                        value: Some("2022-01-01 00:00:00")
                    },
                    Attribute {
                        name: "to",
                        value: Some("123")
                    }
                ],
            })
        );

        let tokens = tokenizer::tokenize(r#"<foo bar>"#, "<", ">");
        assert_eq!(
            parse(&tokens[0]),
            Some(Element {
                name: "foo",
                attrs: vec![
                    Attribute {
                        name: "bar",
                        value: None
                    }
                ],
            })
        );

        let tokens = tokenizer::tokenize("<foo", "<", ">");
        assert_eq!(parse(&tokens[0]), None);

        let tokens = tokenizer::tokenize("foo", "<", ">");
        assert_eq!(parse(&tokens[0]), None);

        let tokens = tokenizer::tokenize("<foo=><bar<><bar>", "<", ">");
        assert_eq!(
            parse(&tokens[0]),
            Some(Element {
                name: "foo",
                attrs: vec![]
            })
        );
        assert_eq!(
            parse(&tokens[1]),
            Some(Element {
                name: "bar<",
                attrs: vec![]
            })
        );
        assert_eq!(
            parse(&tokens[2]),
            Some(Element {
                name: "bar",
                attrs: vec![]
            })
        );

        let tokens = tokenizer::tokenize("<foo bar='baz>", "<", ">");
        assert_eq!(
            parse(&tokens[0]),
            Some(Element {
                name: "foo",
                attrs: vec![Attribute {
                    name: "bar",
                    value: None
                }],
            })
        );

        let tokens = tokenizer::tokenize("<foo bar=123>", "<", ">");
        assert_eq!(
            parse(&tokens[0]),
            Some(Element {
                name: "foo",
                attrs: vec![Attribute {
                    name: "bar",
                    value: None
                }],
            })
        );

        let tokens = tokenizer::tokenize("<foo bar=123 baz='456'>", "<", ">");
        assert_eq!(
            parse(&tokens[0]),
            Some(Element {
                name: "foo",
                attrs: vec![
                    Attribute {
                        name: "bar",
                        value: None
                    },
                    Attribute {
                        name: "baz",
                        value: Some("456")
                    }
                ],
            })
        );
    }
}
