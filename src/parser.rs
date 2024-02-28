use crate::tokenizer;
use crate::element_parser;
use std::collections::HashSet;

#[derive(Debug, PartialEq)]
pub enum ContentPart<'a, 'b, 'c> {
    Element(Element<'a, 'b, 'c>),
    Text(Text<'a, 'b, 'c>),
}

#[derive(Debug, PartialEq)]
pub struct Element<'a, 'b, 'c> {
    pub start_element: element_parser::Element,
    pub start_token: tokenizer::Token<'a, 'b, 'c>,
    pub end_token: tokenizer::Token<'a, 'b, 'c>,
    pub children: Vec<ContentPart<'a, 'b, 'c>>,
}

#[derive(Debug, PartialEq)]
pub struct Text<'a, 'b, 'c> {
    pub token: tokenizer::Token<'a, 'b, 'c>,
}

#[derive(Debug, PartialEq)]
pub struct Content<'a, 'b, 'c> {
    pub raw: &'a str,
    pub parts: Vec<ContentPart<'a, 'b, 'c>>,
}

fn tree<'a, 'b, 'c, 'd>(
    tokens: &'d Vec<tokenizer::Token<'a, 'b, 'c>>,
    cursor: usize,
    parts: &mut Vec<ContentPart<'a, 'b, 'c>>,
    parent_el: Option<&element_parser::Element>,
) -> (usize, Option<&'d tokenizer::Token<'a, 'b, 'c>>) {
    let mut cursor = cursor;

    loop {
        let t = tokens.get(cursor);
        cursor = cursor + 1;

        if t.is_none() {
            break (cursor, None);
        }

        let t: &tokenizer::Token<'a, 'b, 'c> = t.unwrap();

        enum State<'a, 'b, 'c> {
            Closed,
            Content(Vec<ContentPart<'a, 'b, 'c>>),
        }

        let part: State<'a, 'b, 'c> = match t.kind {
            tokenizer::TokenKind::Element(_) => {
                element_parser::parse(t).map_or(
                    State::Content(
                        vec![
                            ContentPart::Text(
                                Text {
                                    token: t.clone()
                                }
                            )
                        ]
                    ),
                    |el| {
                        if let Some(parent_el) = parent_el {
                            if el.name.starts_with("/") && parent_el.name == el.name.trim_start_matches("/") {
                                return State::Closed;
                            }
                        }

                        let mut children = vec![];
                        let (new_cursor, end_token) = tree(tokens, cursor, &mut children, Some(&el));

                        cursor = new_cursor;

                        if let Some(end) = end_token {
                            State::Content(
                                vec![
                                    ContentPart::Element(
                                        Element {
                                            start_element: el,
                                            start_token: t.clone(),
                                            end_token: end.clone(),
                                            children: children,
                                        }
                                    )
                                ]
                            )
                        } else {
                            let mut parts = vec![
                                ContentPart::Text(
                                    Text {
                                        token: t.clone()
                                    }
                                )
                            ];
                            parts.extend(children);

                            State::Content(parts)
                        }
                    }
                )
            },
            _ => State::Content(vec![ContentPart::Text(Text {token: t.clone()})]),
        };

        match part {
            State::Closed => break (cursor, Some(t)),
            State::Content(parsed) => parts.extend(parsed),
        }
    }
}

pub fn parse<'a, 'b, 'c>(content: &'a str, delimiter_start: &'b str, delimiter_end: &'c str) -> Content<'a, 'b, 'c> {
    let tokens: Vec<tokenizer::Token<'a, 'b, 'c>> = tokenizer::tokenize(content, delimiter_start, delimiter_end);
    let mut content_parts: Vec<ContentPart<'a, 'b, 'c>> = vec![];

    tree(&tokens, 0, &mut content_parts, None);

    Content {
        raw: content,
        parts: content_parts,
    }
}

#[test]
fn test_parse() {
    //             0         1          2
    // pos:        012345678901234567 89012345678
    //             |  |            ||||    |   |
    //             0  |      1     |||2    |   |
    // byte_pos:   012345678901234567-01234567890
    let content = "foo<bar baz='13'>あ</bar>fuga";

    assert_eq!(
        parse(content, "<", ">"),
        Content {
            raw: content,
            parts: vec![
                ContentPart::Text(Text {
                    token: tokenizer::Token {
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
                        name: "bar".to_string(),
                        attrs: vec![element_parser::Attribute {
                            name: "baz".to_string(),
                            value: Some("13".to_string())
                        }]
                    },
                    start_token: tokenizer::Token {
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
                    end_token: tokenizer::Token {
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
                    children: vec![
                        ContentPart::Text(Text {
                            token: tokenizer::Token {
                                value: "あ",
                                kind: tokenizer::TokenKind::Text,
                                start: 17,
                                byte_start: 17,
                                end: 18,
                                byte_end: 20,
                            }
                        }),
                    ],
                }),
                ContentPart::Text(Text {
                    token: tokenizer::Token {
                        value: "fuga",
                        kind: tokenizer::TokenKind::Text,
                        start: 24,
                        byte_start: 26,
                        end: 28,
                        byte_end: 30,
                    }
                }),
            ]
        }
    );

    //             0         1         2
    // pos:        0123456789012345678901
    //             |  |           |    |
    //             0  |      1    |    |
    // byte_pos:   0123456789012345679012
    let content = "foo<bar baz='13'>fuga";

    assert_eq!(
        parse(content, "<", ">"),
        Content {
            raw: content,
            parts: vec![
                ContentPart::Text(Text {
                    token: tokenizer::Token {
                        value: "foo",
                        kind: tokenizer::TokenKind::Text,
                        start: 0,
                        byte_start: 0,
                        end: 3,
                        byte_end: 3,
                    }
                }),
                ContentPart::Text(Text {
                    token: tokenizer::Token {
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
                    token: tokenizer::Token {
                        value: "fuga",
                        kind: tokenizer::TokenKind::Text,
                        start: 17,
                        byte_start: 17,
                        end: 21,
                        byte_end: 21,
                    }
                }),
            ]
        }
    );
}