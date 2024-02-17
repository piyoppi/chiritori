use super::Formatter;
pub struct IndentRemover {}

impl Formatter for IndentRemover {
    fn format(&self, content: &mut String, byte_pos: usize) -> usize {
        let mut cursor = byte_pos;

        let bytes = content.as_bytes();
        if bytes[byte_pos] != b'\n' {
            return cursor;
        }

        let found = loop {
            cursor = cursor - 1;

            let current = bytes.get(cursor);

            if content.is_char_boundary(cursor) {
                match current {
                    Some(b' ') => {},
                    Some(b'\t') => {},
                    Some(b'\n') => {
                        cursor = cursor + 1;
                        break true
                    },
                    None => break false,
                    _ => break false,
                };
            }

            if cursor == 0 {
                break false;
            }
        };

        if found {
            content.replace_range(cursor..byte_pos, "");
            return cursor;
        }

        return byte_pos;
    }
}

#[test]
fn test_format() {
    let remover = IndentRemover {};

    // if next char is '\n', remove indent
    //
    //                          10        20        30
    //                 0123456789012345678901234567890123
    //                 |                   ^
    let mut content = "+<div>+    hoge+    +    foo</div>".replace('+', "\n");
    assert_eq!(remover.format(&mut content, 20), 16);
    assert_eq!(
        content,
        "+<div>+    hoge++    foo</div>".replace('+', "\n")
    );

    // if next char is '\n', and previous char is not space, do nothing
    //
    //                          10        20        30
    //                 0123456789012345678901234567890123
    //                 |            ^
    let mut content = "+<div>+hoge++++baz</div>".replace('+', "\n");
    assert_eq!(remover.format(&mut content, 13), 13);
    assert_eq!(
        content,
        "+<div>+hoge++++baz</div>".replace('+', "\n")
    );

    // if next char is not '\n', do nothing
    //
    //                          10
    //                 012345678901
    //                 |      ^
    let mut content = "hoge+  a+baz".replace('+', "\n");
    assert_eq!(remover.format(&mut content, 7), 7);
    assert_eq!(
        content,
        "hoge+  a+baz".replace('+', "\n")
    );

    // if next char is not '\n', do nothing
    //
    //                          10
    //                 012345678901
    //                 | ^
    let mut content = "  +baz".replace('+', "\n");
    assert_eq!(remover.format(&mut content, 2), 2);
    assert_eq!(
        content,
        "  +baz".replace('+', "\n")
    );
}