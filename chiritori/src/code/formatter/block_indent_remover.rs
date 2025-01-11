use super::BlockFormatter;
use crate::code::utils::{
    char_pos_finder::find_next_char_pos,
    line_break_pos_finder::{find_next_line_break_pos, find_prev_line_break_pos},
};
use std::ops::Range;

pub struct BlockIndentRemover {}

impl BlockFormatter for BlockIndentRemover {
    /// Return ranges of indents to be removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::chiritori::code::formatter::BlockFormatter;
    /// let remover = chiritori::code::formatter::block_indent_remover::BlockIndentRemover {};
    /// //         input                output            removed
    /// //  +----------------+    +----------------+    +---------+
    /// //  |  f o o +       |    |  f o o +       |    | foo+    |
    /// //  | [+             |    |  +             |    | +       |
    /// //  |  . . f u g a + |    | [. .]f u g a + |    | fuga+   |
    /// //  |  . . p i y o + | => | [. .]p i y o + | => | piyo+   |
    /// //  |  +]            |    |  +             |    | +       |
    /// //  |  b a r         |    |  b a r         |    | bar     |
    /// //  +----------------+    +----------------+    +---------+
    /// //
    /// //                      10        20
    /// //         pos 01234567890123456789012
    /// //             |   |~~     ~~     |
    /// //             |   |^      ^      |
    /// //             |   |removal ranges|
    /// //             |   |              |
    /// //             |   | block range  |
    /// //             |   |<------------>|
    /// let content = "foo++  fuga+  piyo++bar".replace('+', "\n");
    /// assert_eq!(remover.format(&content, 4, 19), vec![5..7, 12..14]);
    /// ```
    fn format(
        &self,
        content: &str,
        start_byte_pos: usize,
        end_byte_pos: usize,
    ) -> Vec<Range<usize>> {
        let bytes = content.as_bytes();

        let indent_ofs = match find_prev_line_break_pos(content, bytes, start_byte_pos, true) {
            Some(pos) => start_byte_pos - pos - 1,
            None => 0,
        };
        let mut current_pos = start_byte_pos + 1;
        let first_indent_len = get_indent_len(content, current_pos);
        let indent_len = first_indent_len.saturating_sub(indent_ofs);

        let mut positions = vec![];
        while end_byte_pos > current_pos {
            let next_pos =
                find_next_line_break_pos(content, bytes, current_pos, false).map(|v| v + 1);
            match next_pos {
                Some(pos) => {
                    if pos > end_byte_pos {
                        break;
                    }
                    let indent_pos = find_next_char_pos(content, bytes, current_pos);

                    if let Some(indent_pos) = indent_pos {
                        let start = std::cmp::min(current_pos + indent_ofs, indent_pos);
                        let end = std::cmp::min(start + indent_len, indent_pos);

                        if start != end {
                            positions.push(start..end);
                        }
                    }

                    current_pos = pos;
                }
                None => break,
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
        let remover = BlockIndentRemover {};

        //   original       removed       formatted
        // +---------+    +---------+    +---------+
        // | foo+    |    | foo+    |    | foo+    |
        // | <rm>+   |    | +       |    | +       |
        // | if {+   |    | ..fuga+ |    | fuga+   |
        // | ..fuga+ | => | ..piyo+ | => | piyo+   |
        // | ..piyo+ |    | +       |    | +       |
        // | }+      |    | bar     |    | bar     |
        // | </rm>+  |    |         |    |         |
        // | bar     |    |         |    |         |
        // +---------+    +---------+    +---------+
        //
        //                      10        20
        //             012345678901234567890123456
        //             |   ^<>     <>     ^
        let content = "foo++  fuga+  piyo++bar".replace('+', "\n");
        assert_eq!(remover.format(&content, 4, 19), vec![5..7, 12..14]);

        //   original       removed       formatted
        // +----------+    +---------+    +---------+
        // | _foo+    |    | _foo+   |    | _foo+   |
        // | _<rm>+   |    | _+      |    | _+      |
        // | _if {+   |    | __fuga+ |    | _fuga+  |
        // | __fuga+  | => | __piyo+ | => | _piyo+  |
        // | __piyo+  |    | _+      |    | _+      |
        // | _}+      |    | _bar    |    | _bar    |
        // | _</rm>+  |    |         |    |         |
        // | _bar     |    |         |    |         |
        // +----------+    +---------+    +---------+
        //
        //                      10        20
        //             012345678901234567890123456
        //             |     ^ *      *      ^
        let content = "_foo+_+__fuga+__piyo+_+_bar"
            .replace('+', "\n")
            .replace('_', "\t");
        assert_eq!(remover.format(&content, 6, 22), vec![8..9, 15..16]);

        //   original       removed       formatted
        // +---------+    +---------+    +---------+
        // | foo+    |    | foo+    |    | foo+    |
        // | <rm>+   |    | +       |    | +       |
        // | if {+   |    | ..fuga+ |    | fuga+   |
        // | ..fuga+ | => | +       |    | +       |
        // | +       |    | ..piyo+ | => | piyo+   |
        // | ..piyo+ |    | +       |    | +       |
        // | }+      |    | bar     |    | bar     |
        // | </rm>+  |    |         |    |         |
        // | bar+    |    |         |    |         |
        // +---------+    +---------+    +---------+
        //
        //                      10        20
        //             012345678901234567890123456
        //             |   ^<>      <>     ^
        let content = "foo++  fuga++  piyo++bar".replace('+', "\n");
        assert_eq!(remover.format(&content, 4, 20), vec![5..7, 13..15]);

        // If the indentation of the marker is greater than the indentation of the block, it cannot be removed.
        //
        //   original       removed       formatted
        // +---------+    +---------+    +---------+
        // | foo+    |    | foo+    |    | foo+    |
        // | ...<rm>+|    | ...+    |    | ...+    |
        // | if {+   |    | ..fuga+ |    | ..fuga+ |
        // | ..fuga+ | => | +       |    | +       |
        // | +       |    | ..piyo+ | => | piyo+   |
        // | ..piyo+ |    | +       |    | +       |
        // | }+      |    | bar     |    | bar     |
        // | </rm>+  |    |         |    |         |
        // | bar+    |    |         |    |         |
        // +---------+    +---------+    +---------+
        //
        //                      10        20
        //             012345678901234567890123456
        //             |      ^               ^
        let content = "foo+   +  fuga++  piyo++bar".replace('+', "\n");
        assert_eq!(remover.format(&content, 7, 20), vec![]);

        //    original          removed          formatted
        // +------------+    +------------+    +------------+
        // | ...foo+    |    | ...foo+    |    | ...foo+    |
        // | ...<rm>+   |    | ...+       |    | ...+       |
        // | ...if {+   |    | .....fuga+ |    | ...fuga+   |
        // | .....fuga+ | => | ..+        | => | ..+        |
        // | ..+        | => | ...+       | => | ...+       |
        // | ...+       | => | ....+      | => | ...+       |
        // | ....+      | => | .....+     | => | ...+       |
        // | .....+     | => | .....piyo+ | => | ...piyo+   |
        // | .....piyo+ |    | ...+       |    | ...+       |
        // | ...}+      |    | ...bar     |    | ...bar     |
        // | ...</rm>+  |    |            |    |            |
        // | ...bar+    |    |            |    |            |
        // +------------+    +------------+    +------------+
        //
        //                      10        20        30        40        50
        //             012345678901234567890123456789012345678901234567890123456
        //             |      ...^...<>     .. ... ...< ...<>    <>     ...^
        let content = "   foo+   +     fuga+  +   +    +     +     piyo+   +bar".replace('+', "\n");
        assert_eq!(
            remover.format(&content, 10, 52),
            vec![14..16, 31..32, 36..38, 42..44]
        );
    }
}
