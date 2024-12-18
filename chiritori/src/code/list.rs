use super::{
    remover::RemoveMarker,
    utils::line_break_pos_finder::{find_next_line_break_pos, find_prev_line_break_pos},
};

const MARKER_START: &str = "\x1b[32m_start\x1b[0m";
const MARKER_END: &str = "\x1b[32m‾end\x1b[0m";
const START_COLOR: &str = "\x1b[31m";
const START_COLOR_YELLOW: &str = "\x1b[33m";
const END_COLOR: &str = "\x1b[0m";
const MARKER_START_LEN: usize = MARKER_START.len();
const MARKER_END_LEN: usize = MARKER_END.len();
const START_COLOR_LEN: usize = START_COLOR.len();
const START_COLOR_YELLOW_LEN: usize = START_COLOR_YELLOW.len();
const END_COLOR_LEN: usize = END_COLOR.len();

const HEAD_START: &str = "-------- [ ";
const HEAD_END: &str = "--------";
const REMOVAL_HEAD: &str = " ]  Ready  ";
const PENDING_REMOVAL_HEAD: &str = " ] Pending ";

pub fn build_item(content: &str, start: usize, end: usize, is_removal: &bool) -> String {
    if end - start == 0 || content.is_empty() {
        return String::new();
    }

    let bytes = content.as_bytes();
    let line_start = find_prev_line_break_pos(content, bytes, start, false)
        .map(|v| v + 1)
        .unwrap_or(0);
    let line_end_start_pos = find_prev_line_break_pos(content, bytes, end - 1, false)
        .map(|v| v + 1)
        .unwrap_or(0);
    let line_end =
        find_next_line_break_pos(content, bytes, end - 1, false).unwrap_or(content.len());

    let color_start = start;
    // If the end position is a line break (= line_end < end), it is not included.
    let color_end = end.min(line_end);

    let (start_color, start_color_len) = if *is_removal {
        (START_COLOR, START_COLOR_LEN)
    } else {
        (START_COLOR_YELLOW, START_COLOR_YELLOW_LEN)
    };

    let marker_start_ofs_len = start - line_start;
    let marker_end_ofs_len = end - line_end_start_pos - 1;
    let mut result = String::with_capacity(
        marker_start_ofs_len
            + MARKER_START_LEN
            + 1
            + start_color_len
            + (line_end - line_start)
            + END_COLOR_LEN
            + 1
            + marker_end_ofs_len
            + MARKER_END_LEN,
    );

    result.push_str(&" ".repeat(marker_start_ofs_len));
    result.push_str(MARKER_START);
    result.push('\n');
    result.push_str(&content[line_start..color_start]);
    result.push_str(start_color);
    result.push_str(&content[color_start..color_end]);
    result.push_str(END_COLOR);
    result.push_str(&content[color_end..line_end]);
    result.push('\n');
    result.push_str(&" ".repeat(marker_end_ofs_len));
    result.push_str(MARKER_END);

    result
}

pub fn build_list(content: &str, markers: &[(RemoveMarker, bool)]) -> String {
    let mut output: String = markers
        .iter()
        .zip(1..=markers.len())
        .map(|(((range, _), is_removal), idx)| {
            let mut res = String::from("\n");
            res.push_str(HEAD_START);
            res.push_str(&idx.to_string());
            res.push_str(if *is_removal {
                REMOVAL_HEAD
            } else {
                PENDING_REMOVAL_HEAD
            });
            res.push_str(HEAD_END);
            res.push('\n');
            res.push_str(&build_item(content, range.start, range.end, is_removal));

            res
        })
        .collect();

    output.push('\n');

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::ops::Range;

    #[rstest]
    //      0        10
    //      012345678901234567
    #[case("aaa+bbbb+ccc+dddd", 4..11, format!("{}{}+{}{}{}{}{}+{}{}", "", MARKER_START, "", START_COLOR, "bbbb+cc", END_COLOR, "c", " ", MARKER_END))]
    #[case("aaa+bbbb+ccc+dddd", 3..11, format!("{}{}+{}{}{}{}{}+{}{}", "   ", MARKER_START, "aaa", START_COLOR, "+bbbb+cc", END_COLOR, "c", " ", MARKER_END))]
    #[case("aaa+bbbb+ccc+dddd", 2..11, format!("{}{}+{}{}{}{}{}+{}{}", "  ", MARKER_START, "aa", START_COLOR, "a+bbbb+cc", END_COLOR, "c", " ", MARKER_END))]
    #[case("aaa+bbbb+ccc+dddd", 4..12, format!("{}{}+{}{}{}{}{}+{}{}", "", MARKER_START, "", START_COLOR, "bbbb+ccc", END_COLOR, "", "  ", MARKER_END))]
    #[case("aaa+bbbb+ccc+dddd", 4..13, format!("{}{}+{}{}{}{}{}+{}{}", "", MARKER_START, "", START_COLOR, "bbbb+ccc", END_COLOR, "", "   ", MARKER_END))]
    #[case("aaa+bbbb+ccc+dddd", 4..16, format!("{}{}+{}{}{}{}{}+{}{}", "", MARKER_START, "", START_COLOR, "bbbb+ccc+ddd", END_COLOR, "d", "  ", MARKER_END))]
    #[case("abcd", 1..2, format!("{}{}+{}{}{}{}{}+{}{}", " ", MARKER_START, "a", START_COLOR, "b", END_COLOR, "cd", " ", MARKER_END))]
    #[case("", 0..0, "")]
    fn test_build_item(
        #[case] content: &str,
        #[case] range: Range<usize>,
        #[case] expected: String,
    ) {
        let content = content.replace('+', "\n");

        assert_eq!(
            build_item(&content, range.start, range.end, &true),
            expected.replace('+', "\n")
        );
    }

    #[rstest]
    #[case("aaa+bbbb+ccc+dddd", 4..11, format!("{}{}+{}{}{}{}{}+{}{}", "", MARKER_START, "", START_COLOR_YELLOW, "bbbb+cc", END_COLOR, "c", " ", MARKER_END))]
    fn test_build_item_pending_removal_range(
        #[case] content: &str,
        #[case] range: Range<usize>,
        #[case] expected: String,
    ) {
        let content = content.replace('+', "\n");

        assert_eq!(
            build_item(&content, range.start, range.end, &false),
            expected.replace('+', "\n")
        );
    }

    #[test]
    fn test_build_list() {
        //             0123456789012345678
        let content = "aaaa+bbbb+cccc+dddd".replace('+', "\n");
        let markers = [((1..2, None), true), ((7..12, None), false)];

        let expected_item1 = build_item(&content, 1, 2, &true);
        let expected_item2 = build_item(&content, 7, 12, &false);

        assert_eq!(
            build_list(&content, &markers),
            format!(
                "\n{}1{}{}\n{}\n{}2{}{}\n{}\n",
                HEAD_START,
                REMOVAL_HEAD,
                HEAD_END,
                expected_item1,
                HEAD_START,
                PENDING_REMOVAL_HEAD,
                HEAD_END,
                expected_item2
            )
            .replace('+', "\n")
        )
    }
}
