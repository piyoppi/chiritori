pub mod prev_line_break_remover;
pub mod indent_remover;

pub trait Formatter {
    fn format(&self, content: &mut String, byte_pos: usize) -> usize;
}

pub fn format(content: &str, removed_pos: &Vec<usize>, formatters: &Vec<Box<dyn self::Formatter>>) -> String {
    removed_pos
        .iter()
        .rev()
        .fold((content.to_string(), None), |(mut acc, processed_pos), pos| {
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
        }).0
}

#[test]
fn test_format() {
    //                       10        20       30        40        50
    //             01234567890123456789012345678901234567890123456789012345
    //                                 ^                            ^
    let content = "+<div>+    hoge+    +    foo+    bar+    baz+    +</div>".replace('+', "\n");
    let removed_pos = &vec![20, 49];
    assert_eq!(
        format(&content, &removed_pos, &vec![
            Box::new(indent_remover::IndentRemover {}),
            Box::new(prev_line_break_remover::PrevLineBreakRemover {}),
        ]),
      // 0123456789012345678901234567890123456789012345
        "+<div>+    hoge+    foo+    bar+    baz+</div>".replace('+', "\n")
    );

    // RemovePos 26 is unprossed because RemovePos 31 is formatted including RemovePos 26.
    // (If RemovePos 26 is processed, RemovePos 26 is out of range after RemovePos 31 is formatted.)
    //                       10        20       30        40
    //             0123456789012345678901234567890123456789012
    //                                       ^    ^
    let content = "+<div>+    +    +    +    +    +</div>".replace('+', "\n");
    let removed_pos = &vec![26, 31];
    assert_eq!(
        format(&content, &removed_pos, &vec![
            Box::new(prev_line_break_remover::PrevLineBreakRemover {}),
        ]),
      // 0123456789012345678901234567890123456789012345
        "+<div>+    +    +    +    +</div>".replace('+', "\n")
    );

    //                       10        20
    //             012345678901234567890123
    //                          ^
    let content = "+<div>+hoge++++baz</div>".replace('+', "\n");
    let removed_pos = &vec![13];
    assert_eq!(
        format(&content, &removed_pos, &vec![
            Box::new(indent_remover::IndentRemover {}),
            Box::new(prev_line_break_remover::PrevLineBreakRemover {}),
        ]),
        "+<div>+hoge+++baz</div>".replace('+', "\n")
    );
}