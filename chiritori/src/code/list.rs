use crate::code::utils::blank_counter;

use super::{
    remover::RemoveMarker,
    utils::{
        line_break_pos_finder::{find_next_line_break_pos, find_prev_line_break_pos},
        line_map::find_line,
    },
};
use serde::Serialize;
use std::ops::Range;

const MARKER_START: &str = "_start";
const MARKER_END: &str = "‾end";
const MARKER_START_COLOR: &str = "\x1b[32m";
const MARKER_END_COLOR: &str = "\x1b[32m";
const START_COLOR: &str = "\x1b[31m";
const START_COLOR_YELLOW: &str = "\x1b[33m";
const RESET_COLOR: &str = "\x1b[0m";

const HEAD_START: &str = "-------- [ ";
const HEAD_END: &str = "--------";
const REMOVAL_HEAD: &str = " ]  Ready  ";
const PENDING_REMOVAL_HEAD: &str = " ] Pending ";
const LINE_COLUMN_WIDTH: usize = 9;

const TABSPACE: &str = "    ";

pub fn build_pretty_string_item(
    content: &str,
    start: usize,
    end: usize,
    is_removal: bool,
    coloring: bool,
    line_range: Option<(usize, usize)>,
) -> String {
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

    let (marker_start_color, marker_end_color, start_color, reset_color) = if coloring {
        (
            MARKER_START_COLOR,
            MARKER_END_COLOR,
            if is_removal {
                START_COLOR
            } else {
                START_COLOR_YELLOW
            },
            RESET_COLOR,
        )
    } else {
        ("", "", "", "")
    };

    let mut removed = String::with_capacity(
        (line_end - line_start)
            + (content[color_start..color_end].lines().count()
                * (start_color.len() + reset_color.len())),
    );
    removed.push_str(&content[line_start..color_start]);
    removed.push_str(
        &content[color_start..color_end]
            .lines()
            .map(|l| {
                let mut str =
                    String::with_capacity(start_color.len() + l.len() + reset_color.len());
                str.push_str(start_color);
                str.push_str(l);
                str.push_str(reset_color);
                str
            })
            .collect::<Vec<_>>()
            .join("\n"),
    );
    removed.push_str(&content[color_end..line_end]);
    removed.push('\n');

    let (code_block, line_number_ofs) = if let Some(line_range) = line_range {
        let mut line_iter = removed.lines();
        (
            &(line_range.0..=line_range.1)
                .map(|i| {
                    let line_column =
                        format!("{:width$} {}", i, "|", width = LINE_COLUMN_WIDTH - 2);
                    line_iter
                        .next()
                        .map(|l| format!("{line_column}{l}\n"))
                        .unwrap_or("".to_string())
                })
                .collect(),
            LINE_COLUMN_WIDTH,
        )
    } else {
        (&removed, 0)
    };

    let marker_start_ofs_len = start - line_start;
    let marker_start_tab_len = blank_counter::count_tabspace(&content[line_start..start]);
    let marker_end_ofs_len = end - line_end_start_pos - 1;
    let marker_end_tab_len = blank_counter::count_tabspace(&content[line_end_start_pos..end]);
    let mut result = String::with_capacity(
        (marker_start_ofs_len + line_number_ofs - marker_start_tab_len + (marker_start_tab_len * TABSPACE.len())  + marker_start_color.len() + MARKER_START.len() + reset_color.len())    // start marker
        + 1                                                                                                                                                                               // \n
        + removed.len()                                                                                                                                                                   // code block
        + 1                                                                                                                                                                               // \n
        + (marker_end_ofs_len + line_number_ofs - marker_end_tab_len + (marker_end_tab_len * TABSPACE.len()) + marker_end_color.len() + MARKER_END.len() + reset_color.len()), // end marker
    );

    // Print a start marker
    result.push_str(&TABSPACE.to_string().repeat(marker_start_tab_len));
    result.push_str(&" ".repeat(line_number_ofs + marker_start_ofs_len - marker_start_tab_len));
    result.push_str(marker_start_color);
    result.push_str(MARKER_START);
    result.push_str(reset_color);
    result.push('\n');

    // Print a code block
    result.push_str(&code_block.replace("\t", TABSPACE));

    // Print an end marker
    result.push_str(&TABSPACE.to_string().repeat(marker_end_tab_len));
    result.push_str(&" ".repeat(marker_end_ofs_len + line_number_ofs - marker_end_tab_len));
    result.push_str(marker_end_color);
    result.push_str(MARKER_END);
    result.push_str(reset_color);

    result
}

pub fn build_pretty_string(
    content: &str,
    markers: &[(RemoveMarker, bool)],
    line_map: Option<&Vec<usize>>,
) -> String {
    let mut output: String = markers
        .iter()
        .zip(1..=markers.len())
        .map(|(((range, _), is_removal), idx)| {
            let line_range = line_map.map(|m| get_line_range(m, range));

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
            res.push_str(&build_pretty_string_item(
                content,
                range.start,
                range.end,
                *is_removal,
                true,
                line_range,
            ));

            res
        })
        .collect();

    output.push('\n');

    output
}

#[derive(Debug, PartialEq, Serialize)]
pub enum ItemStatus {
    Ready,
    Pending,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct ListItem {
    line_range: Option<(usize, usize)>,
    annotated_code_block: String,
    current_status: ItemStatus,
}

pub fn build_list(
    content: &str,
    markers: &[(RemoveMarker, bool)],
    line_map: Option<&Vec<usize>>,
) -> Vec<ListItem> {
    markers
        .iter()
        .map(|((range, _), is_removal)| {
            let line_range = line_map.map(|m| get_line_range(m, range));
            let text = build_pretty_string_item(
                content,
                range.start,
                range.end,
                *is_removal,
                false,
                line_range,
            );

            ListItem {
                line_range,
                annotated_code_block: text,
                current_status: match is_removal {
                    true => ItemStatus::Ready,
                    false => ItemStatus::Pending,
                },
            }
        })
        .collect()
}

fn get_line_range(line_map: &[usize], range: &Range<usize>) -> (usize, usize) {
    // Subtract one extra line number because of a line break at the end.
    //
    // ex)
    //
    //  line_map                 : [2, 5, 8]
    //  text ('+' is line break) : ab+cd+ef+gh
    //  range                    : 3..=8
    //
    //  expected list:
    //
    //      2 | cd
    //      3 | ef
    //
    // If no subtraction is given, the list would look like this.
    //
    //      2 | cd
    //      3 | ef
    //      4 |             <-- This line is redundant
    (
        find_line(line_map, range.start),
        find_line(line_map, range.end - 1),
    )
}

#[cfg(test)]
mod tests {
    use crate::code::utils::line_map::build_line_map;

    use super::*;
    use rstest::rstest;
    use std::ops::Range;

    const MARKER_START_WITH_COLOR: &str = "\x1b[32m_start\x1b[0m";
    const MARKER_END_WITH_COLOR: &str = "\x1b[32m‾end\x1b[0m";

    #[rstest]
    //      0        10
    //      012345678901234567
    #[case("aaa+bbbb+ccc+dddd", 4..11, format!("{}{}{}{}{}{}{}{}{}{}{}{}{}", MARKER_START_WITH_COLOR, "+", START_COLOR, "bbbb", RESET_COLOR, "+", START_COLOR, "cc", RESET_COLOR, "c", "+", " ", MARKER_END_WITH_COLOR))]
    #[case("aaa+bbbb+ccc+dddd", 3..11, format!("{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}", "   ", MARKER_START_WITH_COLOR, "+", "aaa", START_COLOR, "", RESET_COLOR, "+", START_COLOR, "bbbb", RESET_COLOR, "+", START_COLOR, "cc", RESET_COLOR, "c", "+", " ", MARKER_END_WITH_COLOR))]
    #[case("aaa+bbbb+ccc+dddd", 2..11, format!("{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}", "  ", MARKER_START_WITH_COLOR, "+", "aa", START_COLOR, "a", RESET_COLOR, "+", START_COLOR, "bbbb", RESET_COLOR, "+", START_COLOR, "cc", RESET_COLOR, "c", "+", " ", MARKER_END_WITH_COLOR))]
    #[case("aaa+bbbb+ccc+dddd", 4..12, format!("{}{}{}{}{}{}{}{}{}{}{}{}", MARKER_START_WITH_COLOR, "+",  START_COLOR, "bbbb", RESET_COLOR, "+", START_COLOR,  "ccc", RESET_COLOR, "+", "  ", MARKER_END_WITH_COLOR))]
    #[case("aaa+bbbb+ccc+dddd", 4..13, format!("{}{}{}{}{}{}{}{}{}{}{}{}", MARKER_START_WITH_COLOR, "+", START_COLOR, "bbbb", RESET_COLOR, "+", START_COLOR, "ccc", RESET_COLOR, "+", "   ", MARKER_END_WITH_COLOR))]
    #[case("aaa+bbbb+ccc+dddd", 4..16, format!("{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}", MARKER_START_WITH_COLOR, "+", START_COLOR, "bbbb", RESET_COLOR, "+", START_COLOR, "ccc", RESET_COLOR, "+", START_COLOR, "ddd", RESET_COLOR, "d", "+", "  ", MARKER_END_WITH_COLOR))]
    #[case("abcd", 1..2, format!("{}{}{}{}{}{}{}{}{}{}{}", " ", MARKER_START_WITH_COLOR, "+", "a", START_COLOR, "b", RESET_COLOR, "cd", "+", " ", MARKER_END_WITH_COLOR))]
    #[case("", 0..0, "")]
    fn test_build_item(
        #[case] content: &str,
        #[case] range: Range<usize>,
        #[case] expected: String,
    ) {
        let content = content.replace('+', "\n");

        assert_eq!(
            build_pretty_string_item(&content, range.start, range.end, true, true, None),
            expected.replace('+', "\n")
        );
    }

    #[test]
    fn test_build_item_with_line_number() {
        let content = "aaaaaaa
bbbbbbb
ccccccc
ddddddd
eeeeeee";
        let line_map = build_line_map(content);
        let line_range = (find_line(&line_map, 19), find_line(&line_map, 30));

        #[rustfmt::skip]
        assert_eq!(
            build_pretty_string_item(content, 19, 30, true, true, Some(line_range)),
            format!(
                "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
                "            ", MARKER_START_WITH_COLOR, "+",
                "      3 |", "ccc", START_COLOR, "cccc", RESET_COLOR, "+",
                "      4 |", START_COLOR, "dddddd", RESET_COLOR, "d", "+",
                "              ", MARKER_END_WITH_COLOR
            ).replace('+', "\n")
        );
    }

    #[test]
    fn test_build_item_without_coloring() {
        let content = "aaaaaaa
bbbbbbb
ccccccc
ddddddd
eeeeeee";

        #[rustfmt::skip]
        assert_eq!(
            build_pretty_string_item(content, 19, 30, true, false, None),
            format!(
                "{}{}{}{}{}{}{}{}{}",
                "   ", MARKER_START , "+",
                "ccccccc", "+",
                "ddddddd", "+",
                "     ", MARKER_END
            ).replace('+', "\n")
        );
    }

    #[rstest]
    #[case("aaa+bbbb+ccc+dddd", 4..11, format!("{}{}{}{}{}{}{}{}{}{}{}{}{}", MARKER_START_WITH_COLOR, "+", START_COLOR_YELLOW, "bbbb", RESET_COLOR, "+", START_COLOR_YELLOW, "cc", RESET_COLOR, "c", "+", " ", MARKER_END_WITH_COLOR))]
    fn test_build_item_pending_removal_range(
        #[case] content: &str,
        #[case] range: Range<usize>,
        #[case] expected: String,
    ) {
        let content = content.replace('+', "\n");

        assert_eq!(
            build_pretty_string_item(&content, range.start, range.end, false, true, None),
            expected.replace('+', "\n")
        );
    }

    #[test]
    fn test_build_list() {
        //             0123456789012345678
        let content = "aaaa+bbbb+cccc+dddd".replace('+', "\n");
        let markers = [((1..2, None), true), ((7..14, None), false)];
        let line_map = build_line_map(&content);

        assert_eq!(
            build_list(&content, &markers, Some(&line_map)),
            vec![
                ListItem {
                    line_range: Some((1, 1)),
                    annotated_code_block: "          _start\n      1 |aaaa\n          ‾end"
                        .to_string(),
                    current_status: ItemStatus::Ready
                },
                ListItem {
                    line_range: Some((2, 3)),
                    annotated_code_block:
                        "           _start\n      2 |bbbb\n      3 |cccc\n            ‾end"
                            .to_string(),
                    current_status: ItemStatus::Pending
                },
            ]
        )
    }

    #[test]
    fn test_build_pretty_string() {
        //             0123456789012345678
        let content = "aaaa+bbbb+cccc+dddd".replace('+', "\n");
        let markers = [((1..2, None), true), ((7..12, None), false)];

        let expected_item1 = build_pretty_string_item(&content, 1, 2, true, true, None);
        let expected_item2 = build_pretty_string_item(&content, 7, 12, false, true, None);

        assert_eq!(
            build_pretty_string(&content, &markers, None),
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
