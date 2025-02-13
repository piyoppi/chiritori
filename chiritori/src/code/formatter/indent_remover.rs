use super::Formatter;
pub struct IndentRemover {}

impl Formatter for IndentRemover {
    /// Return the range of blank spaces ( = an indent ) to be deleted.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::chiritori::code::formatter::Formatter;
    /// let remover = chiritori::code::formatter::indent_remover::IndentRemover {};
    /// //           input                      output               removed
    /// //  +---------------------+    +---------------------+    +-----------+
    /// //  |  f o o +            |    |  f o o +            |    | foo+      |
    /// //  |  . . . .]+          | => | [. . . .]+          | => | +         |
    /// //  |  . . . . +          |    |  . . . . +          |    | ....+     |
    /// //  |  . . . . b a r      |    |  . . . . b a r      |    | ....bar   |
    /// //  +---------------------+    +---------------------+    +-----------+
    /// //
    /// //                      10        20
    /// //         pos 01234567890123456789012
    /// //             |   ----
    /// //             |   Removal spaces
    /// let content = "foo+    +    +    bar".replace('+', "\n");
    /// assert_eq!(remover.format(&content, 8), (4, 8));
    /// ```
    fn format(&self, content: &str, byte_pos: usize) -> (usize, usize) {
        let mut cursor = byte_pos;
        let bytes = content.as_bytes();

        if cursor >= bytes.len() || !content.is_char_boundary(cursor) || bytes[cursor] != b'\n' {
            return (byte_pos, byte_pos);
        }

        let found = loop {
            if cursor == 0 {
                break false;
            }

            cursor -= 1;

            let current = bytes.get(cursor);

            if content.is_char_boundary(cursor) {
                match current {
                    Some(b' ') => {}
                    Some(b'\t') => {}
                    Some(b'\n') => {
                        cursor += 1;
                        break true;
                    }
                    None => break false,
                    _ => break false,
                };
            }
        };

        if found {
            return (cursor, byte_pos);
        }

        (byte_pos, byte_pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        let remover = IndentRemover {};

        // if next char is '\n', remove indent
        //
        //                      10        20        30
        //             0123456789012345678901234567890123
        //             |               ^   ^
        let content = "+<div>+    hoge+    +    foo</div>".replace('+', "\n");
        assert_eq!(remover.format(&content, 20), (16, 20));

        // if next char is '\n', and previous char is not space, do nothing
        //
        //                      10        20        30
        //             0123456789012345678901234567890123
        //             |            ^
        let content = "+<div>+hoge++++baz</div>".replace('+', "\n");
        assert_eq!(remover.format(&content, 13), (13, 13));

        // if next char is not '\n', do nothing
        //
        //                      10
        //             012345678901
        //             |      ^
        let content = "hoge+  a+baz".replace('+', "\n");
        assert_eq!(remover.format(&content, 7), (7, 7));

        // if next char is not '\n', do nothing
        //
        //                      10
        //             012345678901
        //             | ^
        let content = "  +baz".replace('+', "\n");
        assert_eq!(remover.format(&content, 2), (2, 2));

        // if char boundary is invalid, do nothing
        //
        //                      10       20        30
        //             0123456789.2345678901234567890123
        //             |         ^
        let content = "+<div>+  あ  +    +    foo</div>".replace('+', "\n");
        assert_eq!(remover.format(&content, 10), (10, 10));

        let content = "".to_string();
        assert_eq!(remover.format(&content, 0), (0, 0));

        let content = "\n".to_string();
        assert_eq!(remover.format(&content, 0), (0, 0));
    }
}
