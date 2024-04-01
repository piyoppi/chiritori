use super::utils::line_break_pos_finder::find_prev_line_break_pos;
use super::Formatter;

pub struct PrevLineBreakRemover {}

impl Formatter for PrevLineBreakRemover {
    fn format(&self, content: &str, byte_pos: usize) -> (usize, usize) {
        let bytes = content.as_bytes();

        let line_break_pos = find_prev_line_break_pos(content, bytes, byte_pos)
            .map(|pos| find_prev_line_break_pos(content, bytes, pos))
            .flatten();

        if let Some(line_break_pos) = line_break_pos {
            (line_break_pos + 1, byte_pos)
        } else {
            (byte_pos, byte_pos)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        let remover = PrevLineBreakRemover {};

        //                      10        20
        //             012345678901234567890123456
        //             |             ^
        let content = "    hoge++    +    foo</div>".replace('+', "\n");
        assert_eq!(remover.format(&content, 14), (9, 14));

        //                      10        20
        //             012345678901234567890123456
        //             |            ^
        let content = "    hoge+    +    foo</div>".replace('+', "\n");
        assert_eq!(remover.format(&content, 13), (13, 13));

        //                      10        20
        //             012345678901234567890123456
        //             |            ^
        let content = "    hoge+  x +    foo</div>".replace('+', "\n");
        let remover = PrevLineBreakRemover {};
        assert_eq!(remover.format(&content, 13), (13, 13));

        //                      10        20
        //             0123456789012345678901234567
        //             |             ^
        let content = "    hoge +    +    foo</div>".replace('+', "\n");
        assert_eq!(remover.format(&content, 14), (14, 14));

        //                      10        20
        //             0123456789012345678901234567
        //             |            ^
        let content = "    hoge +    +    foo</div>".replace('+', "\n");
        assert_eq!(remover.format(&content, 13), (13, 13));

        //                      10
        //             012345678901234567
        //             |      ^
        let content = "+hoge++++baz</div>".replace('+', "\n");
        assert_eq!(remover.format(&content, 7), (6, 7));

        //                      10
        //             01234567890123
        //             |  ^
        let content = "+++++baz</div>".replace('+', "\n");
        assert_eq!(remover.format(&content, 3), (2, 3));

        //                      10
        //             01234567890123
        //             |  ^
        let content = "aaaabaz</div>".replace('+', "\n");
        assert_eq!(remover.format(&content, 3), (3, 3));

        //                     10
        //             012345.890123
        //             |     ^
        let content = "aaa+ あ</div>".replace('+', "\n");
        assert_eq!(remover.format(&content, 7), (7, 7));

        let content = "\n".to_string();
        assert_eq!(remover.format(&content, 0), (0, 0));
    }
}
