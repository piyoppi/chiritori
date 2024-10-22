pub fn find_next_char_pos(content: &str, bytes: &[u8], byte_pos: usize) -> Option<usize> {
    let mut cursor = byte_pos;

    loop {
        if cursor >= bytes.len() || cursor == 0 {
            break None;
        }

        match check(content, bytes, &cursor) {
            CheckResult::Skip => {}
            CheckResult::Found => break Some(cursor),
            CheckResult::None => break None
        }

        cursor += 1;
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
        None => CheckResult::None,
        _ => CheckResult::Found,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_next_line_break_pos() {
        //             012345678901234567890123456789
        //             |      ^  ^   
        let content = "   hoge   fuga   piyo".replace('+', "\n");
        assert_eq!(
            find_next_char_pos(&content, content.as_bytes(), 7),
            Some(10)
        );
    }
}
