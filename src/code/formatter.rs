use super::remover::RemovedMarker;
use std::ops::Range;

pub mod block_indent_remover;
pub mod empty_line_remover;
pub mod indent_remover;
pub mod next_line_break_remover;
pub mod prev_line_break_remover;

pub trait Formatter {
    fn format(&self, content: &str, byte_pos: usize, prev_byte_pos: usize) -> (usize, usize);
}

pub trait BlockFormatter {
    fn format(
        &self,
        content: &str,
        start_byte_pos: usize,
        end_byte_pos: usize,
    ) -> Vec<Range<usize>>;
}

pub fn format(
    content: &str,
    removed_pos: &[RemovedMarker],
    formatters: &[Box<dyn self::Formatter>],
    structure_formatters: &[Box<dyn self::BlockFormatter>],
) -> String {
    let mut ranges: Vec<Range<usize>> = vec![];
    let mut open_structure_remove_range: Vec<Range<usize>> = vec![];

    let removed_pos_iter = removed_pos.iter();
    let mut prev_pos = 0;
    for (pos, pair_idx) in removed_pos_iter {
        let range = format_block(content, *pos, prev_pos, formatters);
        ranges.push(range);

        if let Some(pair_idx) = pair_idx {
            let (pair_start_pos, _) = removed_pos[*pair_idx];
            if *pos < pair_start_pos {
                let ranges = structure_formatters.iter().fold(vec![], |mut v, f| {
                    v.extend(f.format(content, *pos, pair_start_pos));
                    v
                });
                open_structure_remove_range.extend(ranges);
            }
        }
        prev_pos = *pos;
    }

    merge_ranges(&mut ranges, open_structure_remove_range);
    merge_overlapped_ranges(&mut ranges);

    ranges
        .into_iter()
        .rev()
        .fold(content.to_string(), |mut content, range| {
            content.replace_range(range, "");

            content
        })
}

fn format_block(
    content: &str,
    pos: usize,
    prev_pos: usize,
    formatters: &[Box<dyn Formatter>],
) -> Range<usize> {
    formatters.iter().fold(pos..pos, |range, f| {
        let (start, end) = f.format(content, pos, prev_pos);
        let start = std::cmp::max(start.min(range.start), prev_pos);
        let end = std::cmp::max(end.max(range.end), pos);

        start..end
    })
}

fn merge_ranges(ranges: &mut Vec<Range<usize>>, new_ranges: Vec<Range<usize>>) {
    if ranges.is_empty() {
        return;
    }

    let mut cursor = Some(ranges.len() - 1);
    let mut new_ranges = new_ranges;

    while !new_ranges.is_empty() {
        let new_range = new_ranges.pop();

        match new_range {
            Some(new_range) => {
                cursor = match cursor {
                    Some(mut cursor) => loop {
                        let range = &ranges[cursor];
                        if range.start < new_range.start {
                            break Some(cursor);
                        }
                        if cursor == 0 {
                            break None;
                        }
                        cursor -= 1;
                    },
                    None => None,
                };

                match cursor {
                    Some(cursor) => ranges.insert(cursor + 1, new_range),
                    None => ranges.insert(0, new_range),
                }
            }
            None => break,
        }
    }
}

fn merge_overlapped_ranges(ranges: &mut Vec<Range<usize>>) {
    let mut write_cursor = 0;
    for read_cursor in 1..ranges.len() {
        if ranges[write_cursor].end >= ranges[read_cursor].start {
            ranges[write_cursor].end = ranges[write_cursor].end.max(ranges[read_cursor].end)
        } else {
            write_cursor += 1;
            if write_cursor != read_cursor {
                ranges[write_cursor] = ranges[read_cursor].clone()
            }
        }
    }
    ranges.truncate(write_cursor + 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        let strategy: Vec<Box<dyn Formatter>> = vec![
            Box::new(indent_remover::IndentRemover {}),
            Box::new(empty_line_remover::EmptyLineRemover {}),
            Box::new(prev_line_break_remover::PrevLineBreakRemover {}),
            Box::new(next_line_break_remover::NextLineBreakRemover {}),
        ];

        // source:        converted:
        //  1 |<div>         1 |<div>
        //  2 |....hoge      2 |....hoge
        //  3 |....X         3 |....foo
        //  4 |....foo       4 |....bar
        //  5 |....bar       5 |....baz
        //  6 |....baz       6 |
        //  7 |              7 |</div>
        //  8 |....X         8 |
        //  9 |</div>
        //
        //                       10        20       30        40        50
        //             01234567890123456789012345678901234567890123456789012345
        //                                ^                             ^
        let content = "<div>+    hoge+    +    foo+    bar+    baz++    +</div>".replace('+', "\n");
        let removed_pos = [(19, None), (49, None)];
        assert_eq!(
            format(&content, &removed_pos, &strategy, &[]),
            //12345678901234567890123456789012345678901234567
            "<div>+    hoge+    foo+    bar+    baz++</div>".replace('+', "\n")
        );

        // source:        converted:
        //  1 |....hoge      1 |....hoge
        //  2 |....X         2 |....foo
        //  3 |....foo
        //
        //                       10        20
        //             0123456789012345678901
        //                          ^
        let content = "    hoge+    +    foo+".replace('+', "\n");
        let removed_pos = [(13, None)];
        assert_eq!(
            format(&content, &removed_pos, &strategy, &[]),
            //12345678901234567890123456789012345678901234567
            "    hoge+    foo+".replace('+', "\n")
        );

        // source:        converted:
        //  1 |....hoge      1 |....hoge
        //  2 |              2 |
        //  3 |....X         3 |....foo
        //  4 |....foo
        //
        //                       10        20
        //             01234567890123456789012
        //                           ^
        let content = "    hoge++    +    foo+".replace('+', "\n");
        let removed_pos = [(14, None)];
        assert_eq!(
            format(&content, &removed_pos, &strategy, &[]),
            //12345678901234567890123456789012345678901234567
            "    hoge++    foo+".replace('+', "\n")
        );

        // source:        converted:
        //  1 |....hoge      1 |....hoge
        //  2 |....X         2 |
        //  3 |              3 |....foo
        //  4 |....foo
        //
        //                       10        20
        //             01234567890123456789012
        //                           ^
        let content = "    hoge+    ++    foo+".replace('+', "\n");
        let removed_pos = [(14, None)];
        assert_eq!(
            format(&content, &removed_pos, &strategy, &[]),
            //12345678901234567890123456789012345678901234567
            "    hoge++    foo+".replace('+', "\n")
        );

        // source:        converted:
        //  1 |....hoge      1 |....hoge
        //  2 |.             2 |
        //  3 |....X         3 |....foo
        //  4 |.
        //  5 |....foo
        //                       10        20
        //             01234567890123456789012
        //                            ^
        let content = "    hoge+ +    + +    foo+".replace('+', "\n");
        let removed_pos = [(15, None)];
        assert_eq!(
            format(&content, &removed_pos, &strategy, &[]),
            //12345678901234567890123456789012345678901234567
            "    hoge++    foo+".replace('+', "\n")
        );

        // RemovePos 31 is unprossed because it conflicts with RemovePos 26
        //
        //                       10        20       30        40
        //             0123456789012345678901234567890123456789012
        //                                       ^    ^
        let content = "+<div>+    +    +    +    +    +</div>".replace('+', "\n");
        let removed_pos = [(26, None), (31, None)];
        assert_eq!(
            format(
                &content,
                &removed_pos,
                &[Box::new(prev_line_break_remover::PrevLineBreakRemover {}),],
                &[]
            ),
            //123456789012345678901234567890123456789012345
            "+<div>+    +    ++    +</div>".replace('+', "\n")
        );

        //                       10        20
        //             012345678901234567890123
        //                          ^
        let content = "+<div>+hoge++++baz</div>".replace('+', "\n");
        let removed_pos = [(13, None)];
        assert_eq!(
            format(
                &content,
                &removed_pos,
                &[
                    Box::new(indent_remover::IndentRemover {}),
                    Box::new(prev_line_break_remover::PrevLineBreakRemover {}),
                ],
                &[]
            ),
            "+<div>+hoge+++baz</div>".replace('+', "\n")
        );
    }

    #[test]
    fn test_merge_ranges() {
        let mut ranges = vec![1..2, 5..6, 10..15];
        let new_ranges = vec![0..1, 3..4, 9..12];
        merge_ranges(&mut ranges, new_ranges);
        assert_eq!(ranges, vec![0..1, 1..2, 3..4, 5..6, 9..12, 10..15]);
    }

    #[test]
    fn test_merge_overlapped_ranges() {
        let mut ranges = vec![1..5, 2..6, 8..10, 9..12, 15..18, 20..24];
        merge_overlapped_ranges(&mut ranges);
        assert_eq!(ranges, vec![1..6, 8..12, 15..18, 20..24]);

        let mut ranges = vec![1..2, 3..6, 3..4, 9..10];
        merge_overlapped_ranges(&mut ranges);
        assert_eq!(ranges, vec![1..2, 3..6, 9..10]);

        let mut ranges = vec![];
        merge_overlapped_ranges(&mut ranges);
        assert_eq!(ranges, vec![]);
    }
}
