pub mod empty_line_remover;
pub mod indent_remover;
pub mod next_line_break_remover;
pub mod prev_line_break_remover;
pub mod utils;

pub trait Formatter {
    fn format(&self, content: &mut String, byte_pos: usize) -> usize;
}

pub fn format(
    content: &str,
    removed_pos: &Vec<usize>,
    formatters: &Vec<Box<dyn self::Formatter>>,
) -> String {
    removed_pos
        .iter()
        .rev()
        .fold(
            (content.to_string(), None),
            |(mut acc, processed_pos), pos| {
                if let Some(processed_pos) = processed_pos {
                    if *pos >= processed_pos {
                        return (acc, Some(processed_pos));
                    }
                }

                let mut pos = *pos;

                if !acc.is_char_boundary(pos) {
                    panic!("Invalid byte position: {}", pos);
                }

                formatters.iter().for_each(|f| {
                    pos = f.format(&mut acc, pos);
                });

                (acc, Some(pos))
            },
        )
        .0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        let strategy: Vec<Box<dyn Formatter>> = vec![
            Box::new(indent_remover::IndentRemover {}),
            Box::new(empty_line_remover::EmptyLineRemover {}),
            Box::new(prev_line_break_remover::PrevLineBreakRemover {}),
            Box::new(next_line_break_remover::NextLineBreakRemover {}),
        ];

        // source:        converted:
        //  1 |<div>         1 |<div>
        //  2 |....hoge      2 |....hoge
        //  3 |....X         3 |....foo
        //  4 |....foo       4 |....bar
        //  5 |....bar       5 |....baz
        //  6 |....baz       6 |
        //  7 |              7 |</div>
        //  8 |....X         8 |
        //  9 |</div>
        //
        //                       10        20       30        40        50
        //             01234567890123456789012345678901234567890123456789012345
        //                                ^                             ^
        let content = "<div>+    hoge+    +    foo+    bar+    baz++    +</div>".replace('+', "\n");
        let removed_pos = &vec![19, 49];
        assert_eq!(
            format(
                &content,
                &removed_pos,
                &strategy
            ),
            //12345678901234567890123456789012345678901234567
            "<div>+    hoge+    foo+    bar+    baz++</div>".replace('+', "\n")
        );

        // source:        converted:
        //  1 |....hoge      1 |....hoge
        //  2 |....X         2 |....foo
        //  3 |....foo
        //
        //                       10        20
        //             0123456789012345678901
        //                          ^
        let content = "    hoge+    +    foo+".replace('+', "\n");
        let removed_pos = &vec![13];
        assert_eq!(
            format(
                &content,
                &removed_pos,
                &strategy
            ),
            //12345678901234567890123456789012345678901234567
            "    hoge+    foo+".replace('+', "\n")
        );

        // source:        converted:
        //  1 |....hoge      1 |....hoge
        //  2 |              2 |
        //  3 |....X         3 |....foo
        //  4 |....foo
        //
        //                       10        20
        //             01234567890123456789012
        //                           ^
        let content = "    hoge++    +    foo+".replace('+', "\n");
        let removed_pos = &vec![14];
        assert_eq!(
            format(
                &content,
                &removed_pos,
                &strategy
            ),
            //12345678901234567890123456789012345678901234567
            "    hoge++    foo+".replace('+', "\n")
        );

        // source:        converted:
        //  1 |....hoge      1 |....hoge
        //  2 |....X         2 |
        //  3 |              3 |....foo
        //  4 |....foo
        //
        //                       10        20
        //             01234567890123456789012
        //                           ^
        let content = "    hoge+    ++    foo+".replace('+', "\n");
        let removed_pos = &vec![14];
        assert_eq!(
            format(
                &content,
                &removed_pos,
                &strategy
            ),
            //12345678901234567890123456789012345678901234567
            "    hoge++    foo+".replace('+', "\n")
        );

        // source:        converted:
        //  1 |....hoge      1 |....hoge
        //  2 |.             2 |
        //  3 |....X         3 |....foo
        //  4 |.
        //  5 |....foo
        //                       10        20
        //             01234567890123456789012
        //                            ^
        let content = "    hoge+ +    + +    foo+".replace('+', "\n");
        let removed_pos = &vec![15];
        assert_eq!(
            format(
                &content,
                &removed_pos,
                &strategy
            ),
            //12345678901234567890123456789012345678901234567
            "    hoge++    foo+".replace('+', "\n")
        );

        // RemovePos 26 is unprossed because RemovePos 31 is formatted including RemovePos 26.
        // (If RemovePos 26 is processed, RemovePos 26 is out of range after RemovePos 31 is formatted.)
        //                       10        20       30        40
        //             0123456789012345678901234567890123456789012
        //                                       ^    ^
        let content = "+<div>+    +    +    +    +    +</div>".replace('+', "\n");
        let removed_pos = &vec![26, 31];
        assert_eq!(
            format(
                &content,
                &removed_pos,
                &vec![Box::new(prev_line_break_remover::PrevLineBreakRemover {}),]
            ),
            //123456789012345678901234567890123456789012345
            "+<div>+    +    +    ++</div>".replace('+', "\n")
        );

        //                       10        20
        //             012345678901234567890123
        //                          ^
        let content = "+<div>+hoge++++baz</div>".replace('+', "\n");
        let removed_pos = &vec![13];
        assert_eq!(
            format(
                &content,
                &removed_pos,
                &vec![
                    Box::new(indent_remover::IndentRemover {}),
                    Box::new(prev_line_break_remover::PrevLineBreakRemover {}),
                ]
            ),
            "+<div>+hoge+++baz</div>".replace('+', "\n")
        );
    }
}
