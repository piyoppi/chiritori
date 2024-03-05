use crate::element_parser;
use crate::tokenizer;

#[derive(Debug, PartialEq)]
pub enum ContentPart<'a, 'b, 'c, 'd> {
    Element(Element<'a, 'b, 'c, 'd>),
    Text(Text<'a, 'b, 'c, 'd>),
}

#[derive(Debug, PartialEq)]
pub struct Element<'a, 'b, 'c, 'd> {
    pub start_element: element_parser::Element<'a>,
    pub start_token: &'d tokenizer::Token<'a, 'b, 'c>,
    pub end_token: &'d tokenizer::Token<'a, 'b, 'c>,
    pub children: Vec<ContentPart<'a, 'b, 'c, 'd>>,
}

#[derive(Debug, PartialEq)]
pub struct Text<'a, 'b, 'c, 'd> {
    pub token: &'d tokenizer::Token<'a, 'b, 'c>,
}

enum State<'a, 'b, 'c, 'd> {
    Closed(
        (
            &'d tokenizer::Token<'a, 'b, 'c>,
            element_parser::Element<'a>,
        ),
    ),
    Hoisted(
        (
            Vec<ContentPart<'a, 'b, 'c, 'd>>,
            &'d tokenizer::Token<'a, 'b, 'c>,
            element_parser::Element<'a>,
        ),
    ),
    Content(Vec<ContentPart<'a, 'b, 'c, 'd>>),
}

fn tree<'a, 'b, 'c, 'd>(
    tokens: &'a Vec<tokenizer::Token<'a, 'b, 'c>>,
    cursor: usize,
    parts: &mut Vec<ContentPart<'a, 'b, 'c, 'd>>,
    parent_elements: Vec<&element_parser::Element>,
) -> (
    usize,
    Option<(
        &'d tokenizer::Token<'a, 'b, 'c>,
        element_parser::Element<'a>,
    )>,
) {
    let mut cursor = cursor;

    loop {
        let t = tokens.get(cursor);
        cursor = cursor + 1;

        if t.is_none() {
            break (cursor, None);
        }

        let t: &tokenizer::Token<'a, 'b, 'c> = t.unwrap();

        let part: State<'a, 'b, 'c, 'd> = match t.kind {
            tokenizer::TokenKind::Element(_) => element_parser::parse(t).map_or(
                State::Content(vec![ContentPart::Text(Text { token: t })]),
                |el| {
                    if el.name.starts_with("/") {
                        let pair_name = el.name.trim_start_matches("/");
                        if parent_elements
                            .iter()
                            .any(|parent_el| parent_el.name == pair_name)
                        {
                            return State::Closed((t, el));
                        }
                    }

                    let mut next_parent_elements = parent_elements.clone();
                    next_parent_elements.push(&el);
                    let mut children = vec![];
                    let (new_cursor, end_part) =
                        tree(tokens, cursor, &mut children, next_parent_elements);

                    cursor = new_cursor;

                    if let Some((end_token, end_el)) = end_part {
                        if el.name == end_el.name.trim_start_matches("/") {
                            return State::Content(vec![ContentPart::Element(Element {
                                start_element: el,
                                start_token: t,
                                end_token: end_token,
                                children: children,
                            })]);
                        } else {
                            let mut parts = vec![ContentPart::Text(Text { token: t })];
                            parts.extend(children);

                            return State::Hoisted((parts, end_token, end_el));
                        }
                    } else {
                        let mut parts = vec![ContentPart::Text(Text { token: t })];
                        parts.extend(children);

                        State::Content(parts)
                    }
                },
            ),
            _ => State::Content(vec![ContentPart::Text(Text { token: t })]),
        };

        match part {
            State::Closed((t, el)) => break (cursor, Some((t, el))),
            State::Hoisted((parsed, t, el)) => {
                parts.extend(parsed);
                return (cursor, Some((t, el)));
            }
            State::Content(parsed) => parts.extend(parsed),
        }
    }
}

pub fn parse<'a, 'b: 'a, 'c: 'a, 'd: 'a>(
    tokens: &'d Vec<tokenizer::Token<'a, 'b, 'c>>,
) -> Vec<ContentPart<'a, 'b, 'c, 'd>> {
    let mut content_parts: Vec<ContentPart<'a, 'b, 'c, 'd>> = vec![];

    tree(tokens, 0, &mut content_parts, vec![]);

    content_parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        //             0         1          2
        // pos:        012345678901234567 89012345678
        //             |  |            ||||    |   |
        //             0  |      1     |||2    |   |
        // byte_pos:   012345678901234567-01234567890
        let content = "foo<bar baz='13'>あ</bar>fuga";

        assert_eq!(
            parse(&tokenizer::tokenize(content, "<", ">")),
            vec![
                ContentPart::Text(Text {
                    token: &tokenizer::Token {
                        value: "foo",
                        kind: tokenizer::TokenKind::Text,
                        start: 0,
                        byte_start: 0,
                        end: 3,
                        byte_end: 3,
                    }
                }),
                ContentPart::Element(Element {
                    start_element: element_parser::Element {
                        name: "bar",
                        attrs: vec![element_parser::Attribute {
                            name: "baz",
                            value: Some("13")
                        }]
                    },
                    start_token: &tokenizer::Token {
                        value: "<bar baz='13'>",
                        kind: tokenizer::TokenKind::Element(tokenizer::ElementToken {
                            delimiter_start: "<",
                            delimiter_end: ">",
                        }),
                        start: 3,
                        byte_start: 3,
                        end: 17,
                        byte_end: 17,
                    },
                    end_token: &tokenizer::Token {
                        value: "</bar>",
                        kind: tokenizer::TokenKind::Element(tokenizer::ElementToken {
                            delimiter_start: "<",
                            delimiter_end: ">",
                        }),
                        start: 18,
                        byte_start: 20,
                        end: 24,
                        byte_end: 26,
                    },
                    children: vec![ContentPart::Text(Text {
                        token: &tokenizer::Token {
                            value: "あ",
                            kind: tokenizer::TokenKind::Text,
                            start: 17,
                            byte_start: 17,
                            end: 18,
                            byte_end: 20,
                        }
                    }),],
                }),
                ContentPart::Text(Text {
                    token: &tokenizer::Token {
                        value: "fuga",
                        kind: tokenizer::TokenKind::Text,
                        start: 24,
                        byte_start: 26,
                        end: 28,
                        byte_end: 30,
                    }
                }),
            ]
        );

        //             0         1         2
        // pos:        0123456789012345678901
        //             |  |           |    |
        //             0  |      1    |    |
        // byte_pos:   0123456789012345679012
        let content = "foo<bar baz='13'>fuga";

        assert_eq!(
            parse(&tokenizer::tokenize(content, "<", ">")),
            vec![
                ContentPart::Text(Text {
                    token: &tokenizer::Token {
                        value: "foo",
                        kind: tokenizer::TokenKind::Text,
                        start: 0,
                        byte_start: 0,
                        end: 3,
                        byte_end: 3,
                    }
                }),
                ContentPart::Text(Text {
                    token: &tokenizer::Token {
                        value: "<bar baz='13'>",
                        kind: tokenizer::TokenKind::Element(tokenizer::ElementToken {
                            delimiter_start: "<",
                            delimiter_end: ">",
                        }),
                        start: 3,
                        byte_start: 3,
                        end: 17,
                        byte_end: 17,
                    }
                }),
                ContentPart::Text(Text {
                    token: &tokenizer::Token {
                        value: "fuga",
                        kind: tokenizer::TokenKind::Text,
                        start: 17,
                        byte_start: 17,
                        end: 21,
                        byte_end: 21,
                    }
                }),
            ]
        );

        //             01234567890123456789012345
        //             |        ||   |||   ||  |
        let content = "<a b='c' ><  d> < /d></a>";

        assert_eq!(
            parse(&tokenizer::tokenize(content, "<", ">")),
            vec![ContentPart::Element(Element {
                start_element: element_parser::Element {
                    name: "a",
                    attrs: vec![element_parser::Attribute {
                        name: "b",
                        value: Some("c")
                    }]
                },
                start_token: &tokenizer::Token {
                    value: "<a b='c' >",
                    kind: tokenizer::TokenKind::Element(tokenizer::ElementToken {
                        delimiter_start: "<",
                        delimiter_end: ">",
                    }),
                    start: 0,
                    byte_start: 0,
                    end: 10,
                    byte_end: 10,
                },
                end_token: &tokenizer::Token {
                    value: "</a>",
                    kind: tokenizer::TokenKind::Element(tokenizer::ElementToken {
                        delimiter_start: "<",
                        delimiter_end: ">",
                    }),
                    start: 21,
                    byte_start: 21,
                    end: 25,
                    byte_end: 25,
                },
                children: vec![ContentPart::Element(Element {
                    start_element: element_parser::Element {
                        name: "d",
                        attrs: vec![]
                    },
                    start_token: &tokenizer::Token {
                        value: "<  d>",
                        kind: tokenizer::TokenKind::Element(tokenizer::ElementToken {
                            delimiter_start: "<",
                            delimiter_end: ">",
                        }),
                        start: 10,
                        byte_start: 10,
                        end: 15,
                        byte_end: 15,
                    },
                    end_token: &tokenizer::Token {
                        value: "< /d>",
                        kind: tokenizer::TokenKind::Element(tokenizer::ElementToken {
                            delimiter_start: "<",
                            delimiter_end: ">",
                        }),
                        start: 16,
                        byte_start: 16,
                        end: 21,
                        byte_end: 21,
                    },
                    children: vec![ContentPart::Text(Text {
                        token: &tokenizer::Token {
                            value: " ",
                            kind: tokenizer::TokenKind::Text,
                            start: 15,
                            byte_start: 15,
                            end: 16,
                            byte_end: 16,
                        }
                    }),]
                }),],
            }),]
        );

        //             0123456789012345678901234567890
        let content = "<[a b='c']><b c='<[d]>'><[/a]>";
        assert_eq!(
            parse(&tokenizer::tokenize(content, "<[", "]>")),
            vec![ContentPart::Element(Element {
                start_element: element_parser::Element {
                    name: "a",
                    attrs: vec![element_parser::Attribute {
                        name: "b",
                        value: Some("c")
                    }]
                },
                start_token: &tokenizer::Token {
                    value: "<[a b='c']>",
                    kind: tokenizer::TokenKind::Element(tokenizer::ElementToken {
                        delimiter_start: "<[",
                        delimiter_end: "]>",
                    }),
                    start: 0,
                    byte_start: 0,
                    end: 11,
                    byte_end: 11,
                },
                end_token: &tokenizer::Token {
                    value: "<[/a]>",
                    kind: tokenizer::TokenKind::Element(tokenizer::ElementToken {
                        delimiter_start: "<[",
                        delimiter_end: "]>",
                    }),
                    start: 24,
                    byte_start: 24,
                    end: 30,
                    byte_end: 30,
                },
                children: vec![
                    ContentPart::Text(Text {
                        token: &tokenizer::Token {
                            value: "<b c='",
                            kind: tokenizer::TokenKind::Text,
                            start: 11,
                            byte_start: 11,
                            end: 17,
                            byte_end: 17,
                        }
                    }),
                    ContentPart::Text(Text {
                        token: &tokenizer::Token {
                            value: "<[d]>",
                            kind: tokenizer::TokenKind::Element(tokenizer::ElementToken {
                                delimiter_start: "<[",
                                delimiter_end: "]>",
                            }),
                            start: 17,
                            byte_start: 17,
                            end: 22,
                            byte_end: 22,
                        }
                    }),
                    ContentPart::Text(Text {
                        token: &tokenizer::Token {
                            value: "'>",
                            kind: tokenizer::TokenKind::Text,
                            start: 22,
                            byte_start: 22,
                            end: 24,
                            byte_end: 24,
                        }
                    }),
                ],
            }),]
        );
    }
}
