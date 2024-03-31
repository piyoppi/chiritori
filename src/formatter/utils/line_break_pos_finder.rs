pub fn find_next_line_break_pos(content: &str, bytes: &[u8], byte_pos: usize) -> Option<usize> {
    let mut cursor = byte_pos;

    loop {
        if cursor >= bytes.len() || cursor == 0 {
            break None;
        }

        match check(content, bytes, &cursor) {
            CheckResult::Skip => {}
            CheckResult::Found => break Some(cursor),
            CheckResult::None => break None,
        }

        cursor = cursor + 1;
    }
}

pub fn find_prev_line_break_pos(content: &str, bytes: &[u8], byte_pos: usize) -> Option<usize> {
    let mut cursor = byte_pos;

    if cursor == 0 {
        return None;
    }

    loop {
        cursor = cursor - 1;

        if cursor >= bytes.len() || cursor == 0 {
            break None;
        }

        match check(content, bytes, &cursor) {
            CheckResult::Skip => {}
            CheckResult::Found => break Some(cursor),
            CheckResult::None => break None,
        }
    }
}

enum CheckResult {
    Skip,
    Found,
    None,
}

fn check(content: &str, bytes: &[u8], cursor: &usize) -> CheckResult {
    if !content.is_char_boundary(*cursor) {
        return CheckResult::Skip;
    }

    match bytes.get(*cursor) {
        Some(b' ') => CheckResult::Skip,
        Some(b'\n') => CheckResult::Found,
        None => CheckResult::None,
        _ => CheckResult::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_next_line_break_pos() {
        //             012345678901234567890123456789
        //                          ^
        let content = "    hoge+    +  +    foo</div>".replace('+', "\n");
        assert_eq!(
            find_next_line_break_pos(&content, content.as_bytes(), 13),
            Some(13)
        );

        //             012345678901234567890123456789
        //                         ^^
        let content = "    hoge+    +  +    foo</div>".replace('+', "\n");
        assert_eq!(
            find_next_line_break_pos(&content, content.as_bytes(), 12),
            Some(13)
        );

        //             012345678901234567890123456789
        //                            ^^
        let content = "    hoge+    +  +    foo</div>".replace('+', "\n");
        assert_eq!(
            find_next_line_break_pos(&content, content.as_bytes(), 15),
            Some(16)
        );

        //             01234567890123.6789012345678901
        //                              ^
        let content = "    hoge+    あ  +    foo</div>".replace('+', "\n");
        assert_eq!(
            find_next_line_break_pos(&content, content.as_bytes(), 14),
            Some(18)
        );

        //             012345678901234567890123456789
        //                                    ^
        let content = "    hoge+    +  +    foo</div>".replace('+', "\n");
        assert_eq!(
            find_next_line_break_pos(&content, content.as_bytes(), 23),
            None
        );
    }

    #[test]
    fn test_find_prev_line_break_pos() {
        //             012345678901234567890123456789
        //                     ^    ^
        let content = "    hoge+    +  +    foo</div>".replace('+', "\n");
        assert_eq!(
            find_prev_line_break_pos(&content, content.as_bytes(), 13),
            Some(8)
        );

        //             012345678901234567890123456789
        //                          ^^
        let content = "    hoge+    +  +    foo</div>".replace('+', "\n");
        assert_eq!(
            find_prev_line_break_pos(&content, content.as_bytes(), 14),
            Some(13)
        );

        //             012345678901234567890123456789
        //                             ^ ^
        let content = "    hoge+    +  +    foo</div>".replace('+', "\n");
        assert_eq!(
            find_prev_line_break_pos(&content, content.as_bytes(), 18),
            Some(16)
        );

        //             0123456789012345678.0123456789
        //                             ^
        let content = "    hoge+    +  + あ   foo</div>".replace('+', "\n");
        assert_eq!(
            find_prev_line_break_pos(&content, content.as_bytes(), 19),
            None
        );

        //             012345678901234567890123456789
        //                                    ^
        let content = "    hoge+    +  +    foo</div>".replace('+', "\n");
        assert_eq!(
            find_prev_line_break_pos(&content, content.as_bytes(), 23),
            None
        );
    }
}
