pub fn find_next_line_break_pos(
    content: &str,
    bytes: &[u8],
    byte_pos: usize,
    pause_on_char: bool,
) -> Option<usize> {
    let mut cursor = byte_pos;

    loop {
        if cursor >= bytes.len() || cursor == 0 {
            break None;
        }

        match check(content, bytes, &cursor) {
            CheckResult::Skip => {}
            CheckResult::Found => break Some(cursor),
            CheckResult::None => {
                if pause_on_char {
                    break None;
                }
            }
        }

        cursor += 1;
    }
}

pub fn find_prev_line_break_pos(
    content: &str,
    bytes: &[u8],
    byte_pos: usize,
    pause_on_char: bool,
) -> Option<usize> {
    let mut cursor = byte_pos;

    if cursor == 0 {
        return None;
    }

    loop {
        cursor -= 1;

        if cursor >= bytes.len() || cursor == 0 {
            break None;
        }

        match check(content, bytes, &cursor) {
            CheckResult::Skip => {}
            CheckResult::Found => break Some(cursor),
            CheckResult::None => {
                if pause_on_char {
                    break None;
                }
            }
        }
    }
}

#[derive(Debug)]
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
        Some(b'\t') => CheckResult::Skip,
        Some(b'\n') => CheckResult::Found,
        None => CheckResult::None,
        _ => CheckResult::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    //      012345678901234567890123456789
    //                   ^
    #[case("    hoge+    +  +    foo</div>", 13, Some(13))]
    //      012345678901234567890123456789
    //                   ^
    #[case("    hoge+ _ _+  +    foo</div>", 13, Some(13))]
    //      012345678901234567890123456789
    //                  ^^
    #[case("    hoge+    +  +    foo</div>", 12, Some(13))]
    //      012345678901234567890123456789
    //                     ^^
    #[case("    hoge+    +  +    foo</div>", 15, Some(16))]
    //      01234567890123.6789012345678901
    //                       ^
    #[case("    hoge+    あ  +    foo</div>", 14, Some(18))]
    //      012345678901234567890123456789
    //                             ^
    #[case("    hoge+    +  +    foo</div>", 23, None)]
    fn test_find_next_line_break_pos(
        #[case] input: String,
        #[case] pos: usize,
        #[case] expected: Option<usize>,
    ) {
        let content = input.replace('+', "\n").replace('_', "\t");
        assert_eq!(
            find_next_line_break_pos(&content, content.as_bytes(), pos, true),
            expected
        );
    }

    #[rstest]
    //      012345678901234567890123456789
    //              ^    ^
    #[case("    hoge+    +  +    foo</div>", true, 13, Some(8))]
    //      012345678901234567890123456789
    //              ^    ^
    #[case("    hoge+ _ _+  +    foo</div>", true, 13, Some(8))]
    //      012345678901234567890123456789
    //                   ^^
    #[case("    hoge+    +  +    foo</div>", true, 14, Some(13))]
    //      012345678901234567890123456789
    //                      ^ ^
    #[case("    hoge+    +  +    foo</div>", true, 18, Some(16))]
    //      0123456789012345678.0123456789
    //                      ^
    #[case("    hoge+    +  + あ   foo</div>", true, 19, None)]
    //      012345678901234567890123456789
    //                             ^
    #[case("    hoge+    +  +    foo</div>", true, 23, None)]
    //      012345678901234567890123456789
    //                             ^
    #[case("    hoge+    +  +    foo</div>", false, 23, Some(16))]
    //      012345678901234567890123456789
    //                             ^
    #[case("    hoge             foo</div>", true, 23, None)]
    fn test_find_prev_line_break_pos(
        #[case] input: String,
        #[case] pause_on_char: bool,
        #[case] pos: usize,
        #[case] expected: Option<usize>,
    ) {
        let content = input.replace('+', "\n").replace('_', "\t");
        assert_eq!(
            find_prev_line_break_pos(&content, content.as_bytes(), pos, pause_on_char),
            expected
        );
    }
}
