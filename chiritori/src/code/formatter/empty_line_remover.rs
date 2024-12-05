use crate::code::utils::line_break_pos_finder::{
    find_next_line_break_pos, find_prev_line_break_pos,
};

use super::Formatter;

pub struct EmptyLineRemover {}

impl Formatter for EmptyLineRemover {
    /// Return the range of blank spaces to be deleted.
    ///
    /// If the line immediately after the deletion position is a new line and there are no blank lines before or after it,
    /// the new line immediately after the deletion position is deleted.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::chiritori::code::formatter::Formatter;
    /// let remover = chiritori::code::formatter::empty_line_remover::EmptyLineRemover {};
    /// //           input                      output               removed
    /// //  +---------------------+    +---------------------+    +-----------+
    /// //  |  . . . . h o g e +  |    |  . . . . h o g e +  |    | ....hoge+ |
    /// //  | [+                  | => | [+]                 | => | ..foo+    |
    /// //  |  . . f o o +        |    |  . . f o o +        |    |           |
    /// //  +---------------------+    +---------------------+    +-----------+
    /// //
    /// //                      10        20
    /// //         pos 012345678901234
    /// //             |        -
    /// //             |        Removal line break
    /// let content = "    hoge++  foo".replace('+', "\n");
    /// assert_eq!(remover.format(&content, 9), (9, 10));
    ///
    /// //           input           removed (No changed)
    /// //  +---------------------+    +-----------+
    /// //  |  . . . . h o g e +  |    | ....hoge+ |
    /// //  | [+                  | => | +         |
    /// //  |  +                  |    | +         |
    /// //  |  . . f o o +        |    | ..foo+    |
    /// //  +---------------------+    +-----------+
    /// //
    /// //                      10        20
    /// //         pos 01234567890123456789012
    /// let content = "    hoge+++  foo".replace('+', "\n");
    /// assert_eq!(remover.format(&content, 9), (9, 9));
    /// ```
    fn format(&self, content: &str, byte_pos: usize) -> (usize, usize) {
        let bytes = content.as_bytes();

        if !content.is_char_boundary(byte_pos) {
            panic!("Invalid byte position: {}", byte_pos);
        }

        if bytes.get(byte_pos) != Some(&b'\n') {
            return (byte_pos, byte_pos);
        }

        let is_not_next_line_empty = find_next_line_break_pos(content, bytes, byte_pos, true)
            .and_then(|pos| find_next_line_break_pos(content, bytes, pos + 1, true))
            .is_none();
        let is_not_prev_line_empty = find_prev_line_break_pos(content, bytes, byte_pos, true)
            .and_then(|pos| find_prev_line_break_pos(content, bytes, pos, true))
            .is_none();

        if is_not_next_line_empty && is_not_prev_line_empty {
            (byte_pos, byte_pos + 1)
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
        let remover = EmptyLineRemover {};

        //                      10
        //             0123456789012345
        //             |        ^
        let content = "    hoge++  foo".replace('+', "\n");
        assert_eq!(remover.format(&content, 9), (9, 10));

        //                      10
        //             0123456789012345
        //             |         ^
        let content = "    hoge+++  foo".replace('+', "\n");
        assert_eq!(remover.format(&content, 10), (10, 10));

        //                      10
        //             0123456789012345
        //             |        ^
        let content = "    hoge+++  foo".replace('+', "\n");
        assert_eq!(remover.format(&content, 9), (9, 9));

        //                      10
        //             01234567890123456
        //             |         ^
        let content = "    hoge++ +  foo".replace('+', "\n");
        assert_eq!(remover.format(&content, 10), (10, 10));
    }
}
