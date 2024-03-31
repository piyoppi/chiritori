use super::utils::line_break_pos_finder::find_prev_line_break_pos;
use super::Formatter;

pub struct PrevLineBreakRemover {}

impl Formatter for PrevLineBreakRemover {
    fn format(&self, content: &mut String, byte_pos: usize) -> usize {
        let bytes = content.as_bytes();

        let line_break_pos = find_prev_line_break_pos(content, bytes, byte_pos)
            .map(|pos| find_prev_line_break_pos(content, bytes, pos))
            .flatten();

        if let Some(line_break_pos) = line_break_pos {
            let remove_start_pos = line_break_pos + 1;
            content.replace_range(remove_start_pos..byte_pos, "");
            return remove_start_pos;
        }

        byte_pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        let remover = PrevLineBreakRemover {};

        //                          10        20
        //                 012345678901234567890123456
        //                 |             ^
        let mut content = "    hoge++    +    foo</div>".replace('+', "\n");
        assert_eq!(remover.format(&mut content, 14), 9);
        assert_eq!(content, "    hoge++    foo</div>".replace('+', "\n"));

        //                          10        20
        //                 012345678901234567890123456
        //                 |            ^
        let mut content = "    hoge+    +    foo</div>".replace('+', "\n");
        assert_eq!(remover.format(&mut content, 13), 13);
        assert_eq!(content, "    hoge+    +    foo</div>".replace('+', "\n"));

        //                          10        20
        //                 012345678901234567890123456
        //                 |            ^
        let mut content = "    hoge+  x +    foo</div>".replace('+', "\n");
        let remover = PrevLineBreakRemover {};
        assert_eq!(remover.format(&mut content, 13), 13);
        assert_eq!(content, "    hoge+  x +    foo</div>".replace('+', "\n"));

        //                          10        20
        //                 0123456789012345678901234567
        //                 |             ^
        let mut content = "    hoge +    +    foo</div>".replace('+', "\n");
        let remover = PrevLineBreakRemover {};
        assert_eq!(remover.format(&mut content, 14), 14);
        assert_eq!(content, "    hoge +    +    foo</div>".replace('+', "\n"));

        //                          10        20
        //                 0123456789012345678901234567
        //                 |            ^
        let mut content = "    hoge +    +    foo</div>".replace('+', "\n");
        let remover = PrevLineBreakRemover {};
        assert_eq!(remover.format(&mut content, 13), 13);
        assert_eq!(content, "    hoge +    +    foo</div>".replace('+', "\n"));

        //                          10
        //                 012345678901234567
        //                 |      ^
        let mut content = "+hoge++++baz</div>".replace('+', "\n");
        assert_eq!(remover.format(&mut content, 7), 6);
        assert_eq!(content, "+hoge+++baz</div>".replace('+', "\n"));

        //                          10
        //                 01234567890123
        //                 |  ^
        let mut content = "+++++baz</div>".replace('+', "\n");
        assert_eq!(remover.format(&mut content, 3), 2);
        assert_eq!(content, "++++baz</div>".replace('+', "\n"));

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

        let mut content = "\n".to_string();
        assert_eq!(remover.format(&mut content, 0), 0);
    }
}
