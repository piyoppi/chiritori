pub mod marker;
pub mod removal_evaluator;

use crate::element_parser::Element;
use crate::parser;
use crate::parser::ContentPart;
use marker::factory::{create, RemoveStrategies};
use removal_evaluator::RemovalEvaluator;
use std::collections::HashMap;
use std::ops::Range;

type RemoveMarker = (Range<usize>, Option<usize>);
pub type RemovedMarker = (usize, Option<usize>);

type RemovalEvaluators = HashMap<String, Box<dyn RemovalEvaluator>>;

pub struct Remover {
    removal_evaluators: RemovalEvaluators,
    remove_strategies: RemoveStrategies,
}

impl Remover {
    pub fn new(removal_evaluators: RemovalEvaluators, remove_strategies: RemoveStrategies) -> Self {
        Self {
            removal_evaluators,
            remove_strategies,
        }
    }

    pub fn remove(&self, content: Vec<ContentPart>, raw: &str) -> (String, Vec<RemoveMarker>) {
        let markers = self.build_remove_marker(&content);
        let mut new_content = raw.to_string();

        for (marker, _) in markers.iter().rev() {
            new_content.replace_range(marker.clone(), "");
        }

        (new_content, markers)
    }

    pub fn build_remove_marker(&self, contents: &[ContentPart]) -> Vec<RemoveMarker> {
        contents.iter().fold(vec![], |mut acc, c| {
            if let parser::ContentPart::Element(el) = c {
                let marker = if is_skip(&el.start_element) {
                    None
                } else {
                    self.removal_evaluators
                        .get(el.start_element.name)
                        .and_then(|evaluator| match evaluator.is_removal(&el.start_element) {
                            true => create(el, &self.remove_strategies),
                            false => None,
                        })
                        .and_then(|(range, closed_range)| {
                            if !range.is_empty() {
                                Some((range, closed_range))
                            } else {
                                None
                            }
                        })
                };

                let child_markers = self.build_remove_marker(&el.children);

                if let Some((mut marker, pair)) = marker {
                    let mut start_cursor = 0;
                    for (child_marker, _) in &child_markers {
                        if marker.contains(&child_marker.start)
                            || marker.contains(&child_marker.end)
                        {
                            marker = marker.start.min(child_marker.start)
                                ..marker.end.max(child_marker.end)
                        } else {
                            break;
                        }
                        start_cursor += 1;
                    }

                    if let Some(mut end_marker) = pair {
                        let mut end_cursor = child_markers.len();
                        for (child_marker, _) in child_markers.iter().rev() {
                            if end_marker.contains(&child_marker.start)
                                || end_marker.contains(&child_marker.end)
                            {
                                end_marker = end_marker.start.min(child_marker.start)
                                    ..end_marker.end.max(child_marker.end)
                            } else {
                                break;
                            }
                            end_cursor -= 1;
                        }

                        let current = acc.len();
                        acc.push((
                            marker,
                            Some(current + (end_cursor - start_cursor).max(0) + 1),
                        ));
                        if start_cursor < end_cursor {
                            acc.extend(child_markers[start_cursor..end_cursor].to_owned());
                        }
                        acc.push((end_marker, Some(current)));
                    } else {
                        acc.push((marker, None));
                    }
                } else {
                    acc.extend(child_markers);
                }
            }

            acc
        })
    }
}

pub fn get_removed_pos(markers: &[RemoveMarker]) -> Vec<RemovedMarker> {
    markers
        .iter()
        .fold(
            (vec![], 0),
            |(mut positions, mut removed_len), (marker, pair_pos)| {
                positions.push((marker.start - removed_len, *pair_pos));
                removed_len += marker.end - marker.start;
                (positions, removed_len)
            },
        )
        .0
}

fn is_skip(el: &Element) -> bool {
    el.attrs.iter().any(|v| v.name == "skip")
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use std::rc::Rc;

    use marker::{
        availability::{
            range_marker_availability::RangeMarkerAvailability,
            unwrap_block_marker_availability::UnwrapBlockMarkerAvailability,
        },
        builder::{
            range_marker_builder::RangeMarkerBuilder,
            unwrap_block_marker_builder::UnwrapBlockMarkerBuilder,
        },
    };
    use removal_evaluator::time_limited_evaluator::TimeLimitedEvaluator;

    use super::*;
    use crate::tokenizer;

    fn initialize_remove_strategy(content: Rc<String>) -> RemoveStrategies {
        vec![
            (
                Box::new(UnwrapBlockMarkerAvailability::new("unwrap-block")),
                Box::new(UnwrapBlockMarkerBuilder { content }),
            ),
            (
                Box::new(RangeMarkerAvailability::default()),
                Box::new(RangeMarkerBuilder::default()),
            ),
        ]
    }

    fn initialize_removal_evaluators() -> RemovalEvaluators {
        let mut removal_evaluators: RemovalEvaluators = HashMap::new();
        removal_evaluators.insert(
            String::from("tl"),
            Box::new(TimeLimitedEvaluator {
                current_time: chrono::Local::now(),
                time_offset: "+00:00".to_string(),
            }),
        );

        removal_evaluators
    }

    #[rstest]
    //      0         1         2         3        4
    //      012345678901234567890123456789012-5678901234
    #[case("foo<tl to='2000-01-01 00:00:00'>あ</tl>fuga", vec![(3..40, None)])]
    #[case("foo<baz>a", vec![])]
    fn test_remove_marker(#[case] content: &str, #[case] expected: Vec<RemoveMarker>) {
        let mut removal_evaluators: RemovalEvaluators = HashMap::new();
        removal_evaluators.insert(
            String::from("tl"),
            Box::new(TimeLimitedEvaluator {
                current_time: chrono::Local::now(),
                time_offset: "+00:00".to_string(),
            }),
        );
        let tokens = tokenizer::tokenize(content, "<", ">");
        let contents = parser::parse(&tokens);
        let content = Rc::new(content.to_string());
        let remover = Remover::new(removal_evaluators, initialize_remove_strategy(content));
        assert_eq!(remover.build_remove_marker(&contents), expected);
    }

    #[test]
    fn test_remove() {
        let content = Rc::new(
            "
<div>
    hoge
    <!-- tl to='2021-12-31 23:50:00' -->
    <h1>Campaign 1</h1>
    <!-- /tl -->
    foo
    bar
    baz
    <!-- tl to='2022-12-31 23:50:00' -->
    <h1>Campaign 2</h1>
    <!-- /tl -->
</div>
"
            .to_string(),
        );
        let remover = Remover::new(
            initialize_removal_evaluators(),
            initialize_remove_strategy(Rc::clone(&content)),
        );
        let (removed, markers) = remover.remove(
            parser::parse(&tokenizer::tokenize(&content, "<!--", "-->")),
            &content,
        );
        assert_eq!(
            removed,
            "+<div>+    hoge+    +    foo+    bar+    baz+    +</div>+".replace('+', "\n")
        );
        assert_eq!(markers, vec![(20..97, None), (126..203, None)]);

        let content = Rc::new(
            "
hoge
<!-- tl to='2021-12-31 23:50:00' -->
<h1>Campaign 1</h1>
<!-- /tl -->
foo
<!-- tl to='2022-12-31 23:50:00' -->
<h1>Campaign 2</h1>
<!-- /tl -->
"
            .to_string(),
        );
        let remover = Remover::new(
            initialize_removal_evaluators(),
            initialize_remove_strategy(Rc::clone(&content)),
        );
        let (removed, markers) = remover.remove(
            parser::parse(&tokenizer::tokenize(&content, "<!--", "-->")),
            &content,
        );
        assert_eq!(
            removed,
            "
hoge

foo

"
            .to_string()
        );
        assert_eq!(markers, vec![(6..75, None), (80..149, None),]);

        let content = Rc::new(
            "
<!-- tl skip to='2021-12-31 23:50:00' -->
<h1>Campaign 1</h1>
<!-- /tl -->
"
            .to_string(),
        );
        let remover = Remover::new(
            initialize_removal_evaluators(),
            initialize_remove_strategy(Rc::clone(&content)),
        );
        let (removed, _) = remover.remove(
            parser::parse(&tokenizer::tokenize(&content, "<!--", "-->")),
            &content,
        );
        assert_eq!(removed, *content);

        let content = Rc::new(
            "
// --- start ---
/* tl to='2021-12-31 23:50:00' unwrap-block */
if (foo) {
    console.log('abc');
    console.log('def');
}
/* /tl */
// ---  end  ---
"
            .to_string(),
        );
        let remover = Remover::new(
            initialize_removal_evaluators(),
            initialize_remove_strategy(Rc::clone(&content)),
        );
        let (removed, markers) = remover.remove(
            parser::parse(&tokenizer::tokenize(&content, "/*", "*/")),
            &content,
        );
        assert_eq!(
            removed,
            "
// --- start ---

    console.log('abc');
    console.log('def');

// ---  end  ---
"
            .to_string()
        );
        assert_eq!(markers, vec![(18..75, Some(1)), (124..135, Some(0))]);

        // byte_pos:           0        10        20        30        40        50        60        70        80        90       100       110       120       130       140
        //                     0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567
        //                         <-------------------------------------> <------------------------------------------>           <-----> <------------------------------------->
        let content = Rc::new("foo+<tl to='2021-01-01 00:00:00'>+bar+</tl>+<tl to='2000-01-01 00:00:00' unwrap-block>+{+  s1+  s2+}+</tl>+<tl to='2021-01-01 00:00:00'>+bar+</tl>".replace("+", "\n"));
        let remover = Remover::new(
            initialize_removal_evaluators(),
            initialize_remove_strategy(Rc::clone(&content)),
        );
        let (removed, markers) = remover.remove(
            parser::parse(&tokenizer::tokenize(&content, "<", ">")),
            &content,
        );
        assert_eq!(removed, "foo+++  s1+  s2++".to_string().replace("+", "\n"));
        assert_eq!(
            markers,
            vec![
                (4..43, None),
                (44..88, Some(2)),
                (99..106, Some(1)),
                (107..146, None),
            ]
        );

        // byte_pos:           0        10        20        30        40        50        60        70        80        90       100       110       120       130       140       150
        //                     0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123
        //                         <-------------------------------------> <------------------------------------------>        <----------------------------------------->      <----->
        let content = Rc::new("foo+<tl to='2021-01-01 00:00:00'>+bar+</tl>+<tl to='2000-01-01 00:00:00' unwrap-block>+{+  s1+  <tl to='2021-01-01 00:00:00'>+  bar+  </tl>+  s2+}+</tl>".replace("+", "\n"));
        let remover = Remover::new(
            initialize_removal_evaluators(),
            initialize_remove_strategy(Rc::clone(&content)),
        );
        let (removed, markers) = remover.remove(
            parser::parse(&tokenizer::tokenize(&content, "<", ">")),
            &content,
        );
        assert_eq!(
            removed,
            "foo+++  s1+  +  s2+".to_string().replace("+", "\n")
        );
        assert_eq!(
            markers,
            vec![
                (4..43, None),
                (44..88, Some(3)),
                (96..139, None),
                (145..152, Some(1)),
            ]
        );
    }

    #[test]
    fn test_removed_pos() {
        // cursor_pos       :0 1 2 3 4 5 6 7 8 9 A B
        // chars            : Q W E R T Y U I O P @
        // remove_marker    :  <--->   <->   <->
        //                     |       |     |
        //                     |   +---+     |
        //                     |   |   +-----+
        //                     |   |   |
        // removed          : Q|R T|U I|P @
        // result_cursor_pos:  ^   ^   ^
        //                     1   3   5
        let markers = vec![(1..3, None), (5..6, None), (8..9, None)];
        assert_eq!(
            get_removed_pos(&markers),
            vec![(1, None), (3, None), (5, None)]
        );
    }
}
