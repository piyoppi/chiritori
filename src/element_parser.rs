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
        AttrBegin,
        AttrWithNoQuote,
        AttrWithDoubleQuote(usize),
        AttrWithSingleQuote(usize),
        ParseError,
    }

    match &token.kind {
        tokenizer::TokenKind::Element(element) => {
            let target = token
                .value
                .trim_start_matches(&element.delimiter_start)
                .trim_end_matches(&element.delimiter_end);

            let values =
                target
                    .char_indices()
                    .fold((vec![], State::NameBegin), |mut acc, (pos, val)| {
                        match acc.1 {
                            State::NameBegin => match val {
                                ' ' => {}
                                '=' => acc.1 = State::ParseError,
                                '"' => acc.1 = State::ParseError,
                                '\'' => acc.1 = State::ParseError,
                                _ => {
                                    acc.1 = State::Name(pos);
                                }
                            },
                            State::Name(start) => match val {
                                ' ' => {
                                    acc.0.push((&target[start..pos], None));
                                    acc.1 = State::NameEnd;
                                }
                                '=' => {
                                    acc.0.push((&target[start..pos], None));
                                    acc.1 = State::AttrBegin
                                }
                                _ => {}
                            },
                            State::NameEnd => match val {
                                ' ' => {}
                                '=' => acc.1 = State::AttrBegin,
                                _ => {
                                    acc.1 = State::Name(pos);
                                }
                            },
                            State::AttrBegin => match val {
                                ' ' => {}
                                '"' => {
                                    acc.1 = State::AttrWithDoubleQuote(pos + 1);
                                }
                                '\'' => {
                                    acc.1 = State::AttrWithSingleQuote(pos + 1);
                                }
                                _ => acc.1 = State::AttrWithNoQuote,
                            },
                            State::AttrWithDoubleQuote(start) => match val {
                                '"' => {
                                    acc.0.last_mut().unwrap().1 = Some(&target[start..pos]);
                                    acc.1 = State::NameBegin
                                }
                                _ => {}
                            },
                            State::AttrWithSingleQuote(start) => match val {
                                '\'' => {
                                    acc.0.last_mut().unwrap().1 = Some(&target[start..pos]);
                                    acc.1 = State::NameBegin
                                }
                                _ => {}
                            },
                            State::AttrWithNoQuote => match val {
                                ' ' => acc.1 = State::NameBegin,
                                _ => {}
                            },
                            State::ParseError => {}
                        }

                        acc
                    });

            if values.1 == State::ParseError {
                return None;
            }

            let (name, attrs) = if let State::Name(start) = values.1 {
                (&target[start..], vec![])
            } else {
                (
                    values.0[0].0,
                    values.0[1..]
                        .iter()
                        .map(|s| Attribute {
                            name: s.0,
                            value: s.1,
                        })
                        .collect(),
                )
            };

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
