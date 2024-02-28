use crate::tokenizer;

#[derive(Debug, PartialEq)]
pub struct Element {
    pub name: String,
    pub attrs: Vec<Attribute>,
}

#[derive(Debug, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: Option<String>,
}

pub fn parse(token: &tokenizer::Token) -> Option<Element> {
    #[derive(PartialEq)]
    enum State {
        NameBegin,
        Name,
        NameEnd,
        AttrBegin,
        AttrWithDoubleQuote,
        AttrWithSingleQuote,
        ParseError
    }

    match &token.kind {
        tokenizer::TokenKind::Element(element) => {
            let values = token.value
                .trim_start_matches(&element.delimiter_start)
                .trim_end_matches(&element.delimiter_end)
                .chars()
                .fold((vec![], State::NameBegin), |mut acc, val| {
                    match acc.1 {
                        State::NameBegin => {
                            match val {
                                ' ' => {},
                                '=' => acc.1 = State::ParseError,
                                '"' => acc.1 = State::ParseError,
                                '\'' => acc.1 = State::ParseError,
                                _   => {
                                    acc.1 = State::Name;
                                    acc.0.push(("".to_string(), None));
                                    acc.0.last_mut().unwrap().0.push_str(&val.to_string())
                                }
                            }
                        },
                        State::Name => {
                            match val {
                                ' ' => acc.1 = State::NameEnd,
                                '=' => acc.1 = State::AttrBegin,
                                _   => acc.0.last_mut().unwrap().0.push_str(&val.to_string())
                            }
                        },
                        State::NameEnd => {
                            match val {
                                ' ' => {},
                                '=' => acc.1 = State::AttrBegin,
                                _   => {
                                    acc.0.push((val.to_string(), None));
                                    acc.1 = State::Name;
                                }
                            }
                        },
                        State::AttrBegin => {
                            match val {
                                ' '     => {},
                                '"'     => {
                                    acc.0.last_mut().unwrap().1 = Some("".to_string());
                                    acc.1 = State::AttrWithDoubleQuote;
                                },
                                '\''    => {
                                    acc.0.last_mut().unwrap().1 = Some("".to_string());
                                    acc.1 = State::AttrWithSingleQuote;
                                },
                                _       => acc.1 = State::ParseError,
                            }
                        },
                        State::AttrWithDoubleQuote => {
                            match val {
                                '"'     => acc.1 = State::NameBegin,
                                _       => acc.0.last_mut().unwrap().1.as_mut().unwrap().push_str(&val.to_string())
                            }
                        },
                        State::AttrWithSingleQuote => {
                            match val {
                                '\'' => acc.1 = State::NameBegin,
                                _    => acc.0.last_mut().unwrap().1.as_mut().unwrap().push_str(&val.to_string())
                            }
                        },
                        State::ParseError => {}
                    }

                    acc
                });

            if values.1 == State::ParseError {
                return None;
            }

            let name = values.0[0].0.to_string();

            let attrs: Vec<Attribute> = values.0[1..].iter().map(|s| {
                Attribute {
                    name: s.0.to_string(),
                    value: s.1.clone()
                }
            }).collect();

            Some(Element {
                name,
                attrs,
            })
        },
        _ => None,
    }
}

#[test]
fn test_parse() {
    let token = tokenizer::Token {
        kind: tokenizer::TokenKind::Element(
            tokenizer::ElementToken {
                delimiter_start: "<!--",
                delimiter_end: "-->"
            }
        ),
        value: "<!-- hello-world -->",
        start: 0,
        byte_start: 0,
        end: 17,
        byte_end: 18,
    };

     assert_eq!(parse(&token), Some(Element {
        name: "hello-world".to_string(),
         attrs: vec![],
     }));

    let token = tokenizer::Token {
        kind: tokenizer::TokenKind::Element(
            tokenizer::ElementToken {
                delimiter_start: "<!--",
                delimiter_end: "-->"
            }
        ),
        value: "<!-- hello-world from=\"2022-01-01 00:00:00\" to='123' -->",
        start: 0,
        byte_start: 0,
        end: 47,
        byte_end: 48,
    };

    assert_eq!(parse(&token), Some(Element {
        name: "hello-world".to_string(),
        attrs: vec![
            Attribute {
                name: "from".to_string(),
                value: Some("2022-01-01 00:00:00".to_string())
            },
            Attribute {
                name: "to".to_string(),
                value: Some("123".to_string())
            }
        ],
    }));

}