pub fn build_line_map(content: &str) -> Vec<usize> {
    content
        .char_indices()
        .fold(vec![], |mut acc, (byte_pos, c)| {
            if c == '\n' {
                acc.push(byte_pos)
            }

            acc
        })
}

pub fn find_line(line_map: &[usize], needle: usize) -> usize {
    let found = line_map.iter().position(|v| *v > needle);

    if let Some(found) = found {
        found + 1
    } else {
        line_map.len() + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    const CONTENT: &str = "abc+def+efg+hijkl+mnopq";

    #[test]
    fn test_build_line_map() {
        assert_eq!(
            build_line_map(&CONTENT.replace('+', "\n")),
            vec![3, 7, 11, 17]
        );
    }

    #[rstest]
    #[case(1, 1)]
    #[case(9, 3)]
    #[case(20, 5)]
    fn test_find_line(#[case] pos: usize, #[case] expected: usize) {
        let mapped = build_line_map(&CONTENT.replace('+', "\n"));
        assert_eq!(find_line(&mapped, pos), expected);
    }
}
