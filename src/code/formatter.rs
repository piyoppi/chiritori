use super::remover::RemovedMarker;

pub mod empty_line_remover;
pub mod indent_remover;
pub mod next_line_break_remover;
pub mod prev_line_break_remover;
pub mod structure_indent_remover;

pub trait Formatter {
    fn format(&self, content: &str, byte_pos: usize, next_byte_pos: usize) -> (usize, usize);
}

pub trait MultiFormatter {
    fn format(&self, content: &str, byte_pos: usize, next_byte_pos: usize) -> Vec<(usize, usize)>;
}

pub fn format(
    content: &str,
    removed_pos: &[RemovedMarker],
    formatters: &[Box<dyn self::Formatter>],
) -> String {
    let mut removed_pos_iter = removed_pos.iter().rev().peekable();

    let mut content = content.to_string();

    while let Some(pos) = removed_pos_iter.next() {
        let next_pos: usize = removed_pos_iter.peek().map_or(0, |p| match p {
            RemovedMarker::Block(pos) => *pos,
            RemovedMarker::OpenStructure(pos) => *pos.last().unwrap_or(&0)
        });

        match pos {
            RemovedMarker::Block(pos) => {
                format_block(&mut content, *pos, next_pos, formatters);
            },
            RemovedMarker::OpenStructure(pos) => {

            }
        }
    }

    content
}

pub fn format_block(content: &mut String, pos: usize, next_pos: usize, formatters: &[Box<dyn self::Formatter>]) {
    if !content.is_char_boundary(pos) {
        panic!("Invalid byte position: {}", pos);
    }

    formatters.iter().fold(pos, |pos, f| {
        let (start, end) = f.format(content, pos, next_pos);
        let start = std::cmp::max(start, next_pos);
        content.replace_range(start..end, "");

        start
    });
}

pub fn format_open_structure(content: &mut str, pos: Vec<usize>, next_pos: usize, formatters: &[Box<dyn self::Formatter>]) {
    pos.iter().for_each(|v| {

    })
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
        let removed_pos = [RemovedMarker::Block(19), RemovedMarker::Block(49)];
        assert_eq!(
            format(&content, &removed_pos, &strategy),
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
        let removed_pos = [RemovedMarker::Block(13)];
        assert_eq!(
            format(&content, &removed_pos, &strategy),
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
        let removed_pos = [RemovedMarker::Block(14)];
        assert_eq!(
            format(&content, &removed_pos, &strategy),
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
        let removed_pos = [RemovedMarker::Block(14)];
        assert_eq!(
            format(&content, &removed_pos, &strategy),
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
        let removed_pos = [RemovedMarker::Block(15)];
        assert_eq!(
            format(&content, &removed_pos, &strategy),
            //12345678901234567890123456789012345678901234567
            "    hoge++    foo+".replace('+', "\n")
        );

        // RemovePos 31 is unprossed because it conflicts with RemovePos 26
        //
        //                       10        20       30        40
        //             0123456789012345678901234567890123456789012
        //                                       ^    ^
        let content = "+<div>+    +    +    +    +    +</div>".replace('+', "\n");
        let removed_pos = [RemovedMarker::Block(26), RemovedMarker::Block(31)];
        assert_eq!(
            format(
                &content,
                &removed_pos,
                &[Box::new(prev_line_break_remover::PrevLineBreakRemover {}),]
            ),
            //123456789012345678901234567890123456789012345
            "+<div>+    +    ++    +</div>".replace('+', "\n")
        );

        //                       10        20
        //             012345678901234567890123
        //                          ^
        let content = "+<div>+hoge++++baz</div>".replace('+', "\n");
        let removed_pos = [RemovedMarker::Block(13)];
        assert_eq!(
            format(
                &content,
                &removed_pos,
                &[
                    Box::new(indent_remover::IndentRemover {}),
                    Box::new(prev_line_break_remover::PrevLineBreakRemover {}),
                ]
            ),
            "+<div>+hoge+++baz</div>".replace('+', "\n")
        );
    }
}
