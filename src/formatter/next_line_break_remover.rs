use super::utils::line_break_pos_finder::find_next_line_break_pos;
use super::Formatter;
pub struct NextLineBreakRemover {}

impl Formatter for NextLineBreakRemover {
    fn format(&self, content: &mut String, byte_pos: usize) -> usize {
        let bytes = content.as_bytes();

        let line_break_pos = find_next_line_break_pos(content, bytes, byte_pos)
            .map(|pos| find_next_line_break_pos(content, bytes, pos + 1))
            .flatten();

        if let Some(line_break_pos) = line_break_pos {
            content.replace_range(byte_pos..line_break_pos, "");
        }

        byte_pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        let remover = NextLineBreakRemover {};

        //                          10        20
        //                 012345678901234567890123456
        //                 |            ^  ^
        let mut content = "    hoge+    +  +    foo</div>".replace('+', "\n");
        assert_eq!(remover.format(&mut content, 13), 13);
        assert_eq!(content, "    hoge+    +    foo</div>".replace('+', "\n"));

        //                          10        20
        //                 012345678901234567890123456
        //                 |            ^
        let mut content = "    hoge+      +    +    foo</div>".replace('+', "\n");
        assert_eq!(remover.format(&mut content, 13), 13);
        assert_eq!(content, "    hoge+    +    foo</div>".replace('+', "\n"));

        //                          10        20
        //                 012345678901234567890123456
        //                 |            ^
        let mut content = "    hoge+    +    foo</div>".replace('+', "\n");
        assert_eq!(remover.format(&mut content, 13), 13);
        assert_eq!(content, "    hoge+    +    foo</div>".replace('+', "\n"));

        //                          10
        //                 01234567890123456
        //                 |            ^
        let mut content = "    hoge+    +  ".replace('+', "\n");
        assert_eq!(remover.format(&mut content, 13), 13);
        assert_eq!(content, "    hoge+    +  ".replace('+', "\n"));

        //                          10        20
        //                 012345678901234567890123456
        //                 |            ^
        let mut content = "    hoge+    ++++    foo</div>".replace('+', "\n");
        assert_eq!(remover.format(&mut content, 13), 13);
        assert_eq!(content, "    hoge+    +++    foo</div>".replace('+', "\n"));

        //                          10
        //                 01234567890123
        //                 |  ^
        let mut content = "aaaabaz</div>".replace('+', "\n");
        assert_eq!(remover.format(&mut content, 3), 3);
        assert_eq!(content, "aaaabaz</div>".replace('+', "\n"));

        //                          10
        //                 012345.890123
        //                 |     ^
        let mut content = "aaa+ あ</div>".replace('+', "\n");
        assert_eq!(remover.format(&mut content, 7), 7);
        assert_eq!(content, "aaa+ あ</div>".replace('+', "\n"));

        let mut content = "".to_string();
        assert_eq!(remover.format(&mut content, 0), 0);

        let mut content = "\n".to_string();
        assert_eq!(remover.format(&mut content, 0), 0);
    }
}
