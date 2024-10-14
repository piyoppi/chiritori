use crate::code::utils::{char_pos_finder::find_next_char_pos, line_break_pos_finder::find_prev_line_break_pos};

use super::StructureFormatter;

pub struct StructureIndentRemover {}

impl StructureFormatter for StructureIndentRemover {
    fn format(&self, content: &str, end_byte_pos: usize, start_byte_pos: usize) -> Vec<(usize, usize)> {
        let bytes = content.as_bytes();

        let indent_ofs = match find_prev_line_break_pos(content, bytes, end_byte_pos, true) {
            Some(pos) => end_byte_pos - pos - 1,
            None => 0
        };
        let mut current_pos = end_byte_pos - indent_ofs - 1;
        let indent_len = get_indent_len(content, current_pos) - indent_ofs;

        let mut positions = vec![];
        while start_byte_pos < current_pos {
            let indent_start_pos = find_prev_line_break_pos(content, bytes, current_pos, false).map(|v| v + 1);

            match indent_start_pos {
                Some(pos) => {
                    positions.push((pos + indent_ofs, pos + indent_ofs + indent_len - 1));
                    current_pos = pos - 1;
                },
                None => break
            }
        }

        positions
    }
}

fn get_indent_len(content: &str, byte_pos: usize) -> usize {
    let bytes = content.as_bytes();

    find_prev_line_break_pos(content, bytes, byte_pos, false)
        .and_then(|p| find_next_char_pos(content, bytes, p + 1).map(|e| e - p - 1))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        let remover = StructureIndentRemover {};

        //   original       removed       formatted
        // +---------+    +---------+    +---------+
        // | foo+    |    | foo+    |    | foo+    |
        // | <rm>+   |    | +       |    | +       |
        // | if {+   |    | ..fuga+ |    | fuga+   |
        // | ..fuga+ | => | ..piyo+ | => | piyo+   |
        // | ..piyo+ |    | +       |    | +       |
        // | }+      |    | bar     |    | bar     |
        // | </rm>+  |    |         |    |         |
        // | bar+    |    |         |    |         |
        // +---------+    +---------+    +---------+
        //
        //                      10        20
        //             012345678901234567890123456
        //             |   ^<>     <>     ^
        let content = "foo++  fuga+  piyo++bar".replace('+', "\n");
        assert_eq!(remover.format(&content, 19, 4), vec![(12, 13), (5, 6)]);


        //    original          removed          formatted
        // +------------+    +------------+    +------------+
        // | ...foo+    |    | ...foo+    |    | ...foo+    |
        // | ...<rm>+   |    | ...+       |    | ...+       |
        // | ...if {+   |    | .....fuga+ |    | ...fuga+   |
        // | .....fuga+ | => | .....piyo+ | => | ...piyo+   |
        // | .....piyo+ |    | ...+       |    | ...+       |
        // | ...}+      |    | ...bar     |    | ...bar     |
        // | ...</rm>+  |    |            |    |            |
        // | ...bar+    |    |            |    |            |
        // +------------+    +------------+    +------------+
        //
        //                      10        20        30
        //             01234567890123456789012345678901234567
        //             |      ...^...<>     ...<>     ...^
        let content = "   foo+   +     fuga+     piyo+   +bar".replace('+', "\n");
        assert_eq!(remover.format(&content, 34, 10), vec![(24, 25), (14, 15)]);
    }
}
