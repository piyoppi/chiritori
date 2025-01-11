pub fn count(s: &str) -> usize {
    s.chars()
        .fold(0, |acc, v| if v == ' ' { acc + 1 } else { acc })
}

pub fn count_tabspace(s: &str) -> usize {
    s.chars()
        .fold(0, |acc, v| if v == '\t' { acc + 1 } else { acc })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count() {
        assert_eq!(count(" a b "), 3)
    }

    #[test]
    fn test_count_tabspace() {
        assert_eq!(count_tabspace("\ta \t b\t"), 3)
    }
}
