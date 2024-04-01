use super::utils::line_break_pos_finder::find_next_line_break_pos;
use super::Formatter;
pub struct NextLineBreakRemover {}

impl Formatter for NextLineBreakRemover {
    fn format(&self, content: &str, byte_pos: usize) -> (usize, usize) {
        let bytes = content.as_bytes();

        let line_break_pos = find_next_line_break_pos(content, bytes, byte_pos)
            .map(|pos| find_next_line_break_pos(content, bytes, pos + 1))
            .flatten();

        if let Some(line_break_pos) = line_break_pos {
            (byte_pos, line_break_pos)
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
        let remover = NextLineBreakRemover {};

        //                      10        20
        //             012345678901234567890123456
        //             |            ^  ^
        let content = "    hoge+    +  +    foo</div>".replace('+', "\n");
        assert_eq!(remover.format(&content, 13), (13, 16));

        //                      10        20
        //             012345678901234567890123456
        //             |            ^
        let content = "    hoge+      +    +    foo</div>".replace('+', "\n");
        assert_eq!(remover.format(&content, 13), (13, 20));

        //                      10        20
        //             012345678901234567890123456
        //             |            ^
        let content = "    hoge+    +    foo</div>".replace('+', "\n");
        assert_eq!(remover.format(&content, 13), (13, 13));

        //                      10
        //             01234567890123456
        //             |            ^
        let content = "    hoge+    +  ".replace('+', "\n");
        assert_eq!(remover.format(&content, 13), (13, 13));

        //                      10        20
        //             012345678901234567890123456
        //             |            ^
        let content = "    hoge+    ++++    foo</div>".replace('+', "\n");
        assert_eq!(remover.format(&content, 13), (13, 14));

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

        let content = "".to_string();
        assert_eq!(remover.format(&content, 0), (0, 0));

        let content = "\n".to_string();
        assert_eq!(remover.format(&content, 0), (0, 0));
    }
}
