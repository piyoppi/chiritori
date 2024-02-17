use crate::tokenizer;
use crate::element_parser;

#[derive(Debug, PartialEq)]
pub enum ContentKind {
    Element(element_parser::Element),
    Text,
}

#[derive(Debug, PartialEq)]
pub struct ContentPart<'a, 'b, 'c> {
    pub token: tokenizer::Token<'a, 'b, 'c>,
    pub kind: ContentKind
}

#[derive(Debug, PartialEq)]
pub struct Content<'a, 'b, 'c> {
    pub raw: &'a str,
    pub parts: Vec<ContentPart<'a, 'b, 'c>>,
}

pub fn parse<'a, 'b, 'c>(content: &'a str, delimiter_start: &'b str, delimiter_end: &'c str) -> Content<'a, 'b, 'c> {
    let tokens = tokenizer::tokenize(content, delimiter_start, delimiter_end);

    let content_part = tokens.iter().map(|t| {
        ContentPart {
            token: t.clone(),
            kind: match t.kind {
                tokenizer::TokenKind::Element(_) => {
                    match element_parser::parse(t.clone()) {
                        Some(e) => ContentKind::Element(e),
                        None => ContentKind::Text,
                    }
                },
                _ => ContentKind::Text,
            }
        }
    }).collect();

    Content {
        raw: content,
        parts: content_part,
    }
}

#[test]
fn test_parse() {
    let content = "
<div>
  <!-- time-limited to='2021-12-31' -->
  <h1>Hello, World!</h1>
</div>";

    assert_eq!(
        parse(content, "<!--", "-->"),
        Content {
            raw: content,
            parts: vec![
                ContentPart {
                    token: tokenizer::Token {
                        kind: tokenizer::TokenKind::Text,
                        value: "\n<div>\n  ",
                        start: 0,
                        byte_start: 0,
                        end: 9,
                        byte_end: 9,
                    },
                    kind: ContentKind::Text,
                },
                ContentPart {
                    token: tokenizer::Token {
                        kind: tokenizer::TokenKind::Element(
                            tokenizer::ElementToken {
                                delimiter_start: "<!--",
                                delimiter_end: "-->",
                            }
                        ),
                        value: "<!-- time-limited to='2021-12-31' -->",
                        start: 9,
                        byte_start: 9,
                        end: 46,
                        byte_end: 46,
                    },
                    kind: ContentKind::Element(
                        element_parser::Element {
                            name: "time-limited".to_string(),
                            attrs: vec![
                                element_parser::Attribute {
                                    name: "to".to_string(),
                                    value: Some("2021-12-31".to_string()),
                                },
                            ],
                        }
                    ),
                },
                ContentPart {
                    token: tokenizer::Token {
                        kind: tokenizer::TokenKind::Text,
                        value: "\n  <h1>Hello, World!</h1>\n</div>",
                        start: 46,
                        byte_start: 46,
                        end: 78,
                        byte_end: 78,
                    },
                    kind: ContentKind::Text,
                },
            ]
        }
    )
}