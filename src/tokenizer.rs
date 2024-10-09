use std::str::Chars;

#[derive(PartialEq, Debug, Clone)]
pub enum TokenKind<'a, 'b> {
    Element(ElementToken<'a, 'b>),
    Text,
}

#[derive(PartialEq, Debug, Clone)]
pub struct ElementToken<'a, 'b> {
    pub delimiter_start: &'a str,
    pub delimiter_end: &'b str,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Token<'a, 'b, 'c> {
    pub kind: TokenKind<'b, 'c>,
    pub value: &'a str,
    pub start: usize,
    pub byte_start: usize,
    pub end: usize,
    pub byte_end: usize,
}

enum State<'a, 'b> {
    Text,
    DelimiterStart(Chars<'a>),
    InDelimiter,
    DelimiterEnd(Chars<'b>),
}

pub fn tokenize<'a, 'b, 'c>(
    source: &'a str,
    delimiter_start: &'b str,
    delimiter_end: &'c str,
) -> Vec<Token<'a, 'b, 'c>> {
    let (mut tokens, state, byte_start_pos, start_pos, current) = source.char_indices().fold(
        (vec![], State::Text, 0, 0, 0),
        |(mut tokens, state, mut byte_start_pos, mut start_pos, current): (
            Vec<Token<'a, 'b, 'c>>,
            State,
            usize,
            usize,
            usize,
        ),
         (byte_pos, c)| {
            let (token_kind, next_state) = get_state(&c, delimiter_start, delimiter_end, state);

            if let Some(token_kind) = token_kind {
                if (byte_pos - byte_start_pos) > 0 {
                    tokens.push(Token {
                        value: &source[byte_start_pos..byte_pos],
                        kind: token_kind,
                        start: start_pos,
                        byte_start: byte_start_pos,
                        end: current,
                        byte_end: byte_pos,
                    });
                }

                start_pos = current;
                byte_start_pos = byte_pos;
            };

            (tokens, next_state, byte_start_pos, start_pos, current + 1)
        },
    );

    let (token_kind, _) = get_state(&' ', delimiter_start, delimiter_end, state);

    let last_byte_pos = source.char_indices().last();
    let additional_token = match last_byte_pos {
        Some((byte_pos, _)) => match token_kind {
            None => Some(Token {
                value: &source[byte_start_pos..],
                kind: TokenKind::Text,
                start: start_pos,
                byte_start: byte_start_pos,
                end: current,
                byte_end: byte_pos + 1,
            }),
            _ => Some(Token {
                value: &source[byte_start_pos..byte_pos + 1],
                kind: token_kind.unwrap(),
                start: start_pos,
                byte_start: byte_start_pos,
                end: current,
                byte_end: byte_pos + 1,
            }),
        },
        None => None,
    };

    if let Some(token) = additional_token {
        tokens.push(token);
    }

    tokens.into_iter().fold(vec![], |mut acc, cur| {
        let merged = {
            let last_token = acc.last_mut();
            match last_token {
                Some(last_token) => {
                    if last_token.kind == TokenKind::Text && cur.kind == TokenKind::Text {
                        last_token.value = &source[last_token.byte_start..cur.byte_end];
                        last_token.end = cur.end;
                        last_token.byte_end = cur.byte_end;

                        true
                    } else {
                        false
                    }
                }
                None => false,
            }
        };

        if !merged {
            acc.push(cur);
        }

        acc
    })
}

fn check_delimiter_start<'a, 'b>(c: &char, delimiter_start: &'a str) -> State<'a, 'b> {
    let mut delimiter_start_chars = delimiter_start.chars();

    if *c == delimiter_start_chars.next().unwrap() {
        State::DelimiterStart(delimiter_start_chars)
    } else {
        State::Text
    }
}

fn get_state<'a, 'b, 'c>(
    c: &char,
    delimiter_start: &'b str,
    delimiter_end: &'c str,
    state: State<'b, 'c>,
) -> (Option<TokenKind<'b, 'c>>, State<'b, 'c>) {
    match state {
        State::Text => match check_delimiter_start(c, delimiter_start) {
            State::DelimiterStart(delimiter_start_chars) => (
                Some(TokenKind::Text),
                State::DelimiterStart(delimiter_start_chars),
            ),
            _ => (None, State::Text),
        },
        State::DelimiterStart(mut current_chars) => {
            let current_char = current_chars.next();

            match current_char {
                Some(current_char) => {
                    if *c == current_char {
                        (None, State::DelimiterStart(current_chars))
                    } else {
                        (None, State::Text)
                    }
                }
                None => (None, State::InDelimiter),
            }
        }
        State::InDelimiter => {
            let mut delimiter_end_chars = delimiter_end.chars();
            if *c == delimiter_end_chars.next().unwrap() {
                (None, State::DelimiterEnd(delimiter_end_chars))
            } else {
                (None, state)
            }
        }
        State::DelimiterEnd(mut current_chars) => {
            let current_char = current_chars.next();

            match current_char {
                Some(current_char) => {
                    if *c == current_char {
                        (None, State::DelimiterEnd(current_chars))
                    } else {
                        (None, State::InDelimiter)
                    }
                }
                None => (
                    Some(TokenKind::Element(ElementToken {
                        delimiter_start,
                        delimiter_end,
                    })),
                    check_delimiter_start(c, delimiter_start),
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let default_element = ElementToken {
            delimiter_start: "[",
            delimiter_end: "]",
        };
        assert_eq!(
            tokenize("r", "[", "]"),
            vec![Token {
                value: "r",
                kind: TokenKind::Text,
                start: 0,
                byte_start: 0,
                end: 1,
                byte_end: 1
            }]
        );
        assert_eq!(
            tokenize("[r]", "[", "]"),
            vec![Token {
                value: "[r]",
                kind: TokenKind::Element(default_element.clone()),
                start: 0,
                byte_start: 0,
                end: 3,
                byte_end: 3
            }]
        );
        assert_eq!(
            tokenize("[r]", "[", "]"),
            vec![Token {
                value: "[r]",
                kind: TokenKind::Element(default_element.clone()),
                start: 0,
                byte_start: 0,
                end: 3,
                byte_end: 3
            }]
        );
        assert_eq!(
            tokenize("aaa[r]g", "[", "]"),
            vec![
                Token {
                    value: "aaa",
                    kind: TokenKind::Text,
                    start: 0,
                    byte_start: 0,
                    end: 3,
                    byte_end: 3
                },
                Token {
                    value: "[r]",
                    kind: TokenKind::Element(default_element.clone()),
                    start: 3,
                    byte_start: 3,
                    end: 6,
                    byte_end: 6
                },
                Token {
                    value: "g",
                    kind: TokenKind::Text,
                    start: 6,
                    byte_start: 6,
                    end: 7,
                    byte_end: 7
                }
            ]
        );
        assert_eq!(
            tokenize("[r]g", "[", "]"),
            vec![
                Token {
                    value: "[r]",
                    kind: TokenKind::Element(default_element.clone()),
                    start: 0,
                    byte_start: 0,
                    end: 3,
                    byte_end: 3
                },
                Token {
                    value: "g",
                    kind: TokenKind::Text,
                    start: 3,
                    byte_start: 3,
                    end: 4,
                    byte_end: 4
                }
            ]
        );
        assert_eq!(
            tokenize("[r]g[r]", "[", "]"),
            vec![
                Token {
                    value: "[r]",
                    kind: TokenKind::Element(default_element.clone()),
                    start: 0,
                    byte_start: 0,
                    end: 3,
                    byte_end: 3
                },
                Token {
                    value: "g",
                    kind: TokenKind::Text,
                    start: 3,
                    byte_start: 3,
                    end: 4,
                    byte_end: 4
                },
                Token {
                    value: "[r]",
                    kind: TokenKind::Element(default_element.clone()),
                    start: 4,
                    byte_start: 4,
                    end: 7,
                    byte_end: 7
                }
            ]
        );
        assert_eq!(
            tokenize("[r][r]", "[", "]"),
            vec![
                Token {
                    value: "[r]",
                    kind: TokenKind::Element(default_element.clone()),
                    start: 0,
                    byte_start: 0,
                    end: 3,
                    byte_end: 3
                },
                Token {
                    value: "[r]",
                    kind: TokenKind::Element(default_element.clone()),
                    start: 3,
                    byte_start: 3,
                    end: 6,
                    byte_end: 6
                }
            ]
        );
        assert_eq!(
            tokenize("[r]こんにちは[r]b", "[", "]"),
            vec![
                Token {
                    value: "[r]",
                    kind: TokenKind::Element(default_element.clone()),
                    start: 0,
                    byte_start: 0,
                    end: 3,
                    byte_end: 3
                },
                Token {
                    value: "こんにちは",
                    kind: TokenKind::Text,
                    start: 3,
                    byte_start: 3,
                    end: 8,
                    byte_end: 18
                },
                Token {
                    value: "[r]",
                    kind: TokenKind::Element(default_element.clone()),
                    start: 8,
                    byte_start: 18,
                    end: 11,
                    byte_end: 21
                },
                Token {
                    value: "b",
                    kind: TokenKind::Text,
                    start: 11,
                    byte_start: 21,
                    end: 12,
                    byte_end: 22
                }
            ]
        );
        assert_eq!(
            tokenize("r[g", "[", "]"),
            vec![Token {
                value: "r[g",
                kind: TokenKind::Text,
                start: 0,
                byte_start: 0,
                end: 3,
                byte_end: 3
            }]
        );
        assert_eq!(
            tokenize(&"<!--r-->gb", "<!--", "-->"),
            vec![
                Token {
                    value: "<!--r-->",
                    kind: TokenKind::Element(ElementToken {
                        delimiter_start: "<!--",
                        delimiter_end: "-->"
                    }),
                    start: 0,
                    byte_start: 0,
                    end: 8,
                    byte_end: 8
                },
                Token {
                    value: "gb",
                    kind: TokenKind::Text,
                    start: 8,
                    byte_start: 8,
                    end: 10,
                    byte_end: 10
                }
            ]
        );
        assert_eq!(
            tokenize("foo<div><!--r-->gb", "<!--", "-->"),
            vec![
                Token {
                    value: "foo<div>",
                    kind: TokenKind::Text,
                    start: 0,
                    byte_start: 0,
                    end: 8,
                    byte_end: 8
                },
                Token {
                    value: "<!--r-->",
                    kind: TokenKind::Element(ElementToken {
                        delimiter_start: "<!--",
                        delimiter_end: "-->"
                    }),
                    start: 8,
                    byte_start: 8,
                    end: 16,
                    byte_end: 16
                },
                Token {
                    value: "gb",
                    kind: TokenKind::Text,
                    start: 16,
                    byte_start: 16,
                    end: 18,
                    byte_end: 18
                }
            ]
        );
    }
}
