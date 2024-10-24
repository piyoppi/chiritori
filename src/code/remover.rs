pub mod marker;
pub mod removal_evaluator;
pub mod time_limited_evaluator;

use crate::parser;
use crate::parser::ContentPart;
use marker::factory::{create, RemoveStrategies};
use removal_evaluator::RemovalEvaluator;
use std::collections::HashMap;
use std::ops::Range;

type RemoveMarker = (Range<usize>, Option<usize>);
pub type RemovedMarker = (usize, Option<usize>);

pub fn remove(
    content: Vec<ContentPart>,
    raw: &str,
    builder_map: &HashMap<&str, Box<dyn RemovalEvaluator>>,
    remove_strategy_map: &RemoveStrategies,
) -> (String, Vec<RemoveMarker>) {
    let markers = build_remove_marker(&content, builder_map, remove_strategy_map);
    let mut new_content = raw.to_string();

    for (marker, _) in markers.iter().rev() {
        new_content.replace_range(marker.clone(), "");
    }

    (new_content, markers)
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

fn build_remove_marker(
    contents: &[ContentPart],
    evaluator_map: &HashMap<&str, Box<dyn RemovalEvaluator>>,
    remove_strategy_map: &RemoveStrategies,
) -> Vec<RemoveMarker> {
    contents.iter().fold(vec![], |mut acc, c| {
        if let parser::ContentPart::Element(el) = c {
            let evaluator = evaluator_map.get(el.start_element.name);
            let marker =
                evaluator.and_then(|evaluator| match evaluator.is_removal(&el.start_element) {
                    true => create(el, remove_strategy_map),
                    false => None,
                });

            let child_markers =
                build_remove_marker(&el.children, evaluator_map, remove_strategy_map);

            if let Some((mut marker, pair)) = marker {
                let mut start_cursor = 0;
                for (child_marker, _) in &child_markers {
                    if marker.contains(&child_marker.start) || marker.contains(&child_marker.end) {
                        marker =
                            marker.start.min(child_marker.start)..marker.end.max(child_marker.end)
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

#[cfg(test)]
mod tests {
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
    use time_limited_evaluator::TimeLimitedEvaluator;

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

    #[test]
    fn test_remove_marker() {
        let mut builder_map: HashMap<&str, Box<dyn RemovalEvaluator>> = HashMap::new();
        builder_map.insert(
            "tl",
            Box::new(TimeLimitedEvaluator {
                current_time: chrono::Local::now(),
                time_offset: "+00:00".to_string(),
            }),
        );
        //                    0         1         2         3        4
        // byte_pos:          012345678901234567890123456789012-5678901234
        let content = Rc::new("foo<tl to='2000-01-01 00:00:00'>あ</tl>fuga".to_string());
        let tokens = tokenizer::tokenize(&content, "<", ">");
        let contents = parser::parse(&tokens);
        assert_eq!(
            build_remove_marker(
                &contents,
                &builder_map,
                &initialize_remove_strategy(Rc::clone(&content))
            ),
            vec![(3..40, None)]
        );

        let content = Rc::new("foo<baz>a".to_string());
        let tokens = tokenizer::tokenize(&content, "<", ">");
        let contents = parser::parse(&tokens);
        assert_eq!(
            build_remove_marker(
                &contents,
                &builder_map,
                &initialize_remove_strategy(Rc::clone(&content))
            ),
            vec![]
        );
    }

    #[test]
    fn test_remove() {
        let mut builder_map: HashMap<&str, Box<dyn RemovalEvaluator>> = HashMap::new();
        builder_map.insert(
            "time-limited",
            Box::new(TimeLimitedEvaluator {
                current_time: chrono::Local::now(),
                time_offset: "+00:00".to_string(),
            }),
        );
        builder_map.insert(
            "tl",
            Box::new(TimeLimitedEvaluator {
                current_time: chrono::Local::now(),
                time_offset: "+00:00".to_string(),
            }),
        );

        let content = Rc::new(
            "
<div>
    hoge
    <!-- time-limited to='2021-12-31 23:50:00' -->
    <h1>Campaign 1</h1>
    <!-- /time-limited -->
    foo
    bar
    baz
    <!-- time-limited to='2022-12-31 23:50:00' -->
    <h1>Campaign 2</h1>
    <!-- /time-limited -->
</div>
"
            .to_string(),
        );
        let (removed, markers) = remove(
            parser::parse(&tokenizer::tokenize(&content, "<!--", "-->")),
            &content,
            &builder_map,
            &initialize_remove_strategy(Rc::clone(&content)),
        );
        assert_eq!(
            removed,
            "+<div>+    hoge+    +    foo+    bar+    baz+    +</div>+".replace('+', "\n")
        );
        assert_eq!(markers, vec![(20..117, None), (146..243, None)]);

        let content = Rc::new(
            "
hoge
<!-- time-limited to='2021-12-31 23:50:00' -->
<h1>Campaign 1</h1>
<!-- /time-limited -->
foo
<!-- time-limited to='2022-12-31 23:50:00' -->
<h1>Campaign 2</h1>
<!-- /time-limited -->
"
            .to_string(),
        );

        let (removed, markers) = remove(
            parser::parse(&tokenizer::tokenize(&content, "<!--", "-->")),
            &content,
            &builder_map,
            &initialize_remove_strategy(Rc::clone(&content)),
        );
        assert_eq!(
            removed,
            "
hoge

foo

"
            .to_string()
        );
        assert_eq!(markers, vec![(6..95, None), (100..189, None),]);

        let content = Rc::new(
            "
// --- start ---
/* time-limited to='2021-12-31 23:50:00' unwrap-block */
if (foo) {
    console.log('abc');
    console.log('def');
}
/* /time-limited */
// ---  end  ---
"
            .to_string(),
        );

        let (removed, markers) = remove(
            parser::parse(&tokenizer::tokenize(&content, "/*", "*/")),
            &content,
            &builder_map,
            &initialize_remove_strategy(Rc::clone(&content)),
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
        assert_eq!(markers, vec![(18..85, Some(1)), (134..155, Some(0))]);

        // byte_pos:           0        10        20        30        40        50        60        70        80        90       100       110       120       130       140
        //                     0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567
        //                         <-------------------------------------> <------------------------------------------>           <-----> <------------------------------------->
        let content = Rc::new("foo+<tl to='2021-01-01 00:00:00'>+bar+</tl>+<tl to='2000-01-01 00:00:00' unwrap-block>+{+  s1+  s2+}+</tl>+<tl to='2021-01-01 00:00:00'>+bar+</tl>".replace("+", "\n"));
        let (removed, markers) = remove(
            parser::parse(&tokenizer::tokenize(&content, "<", ">")),
            &content,
            &builder_map,
            &initialize_remove_strategy(Rc::clone(&content)),
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
        let (removed, markers) = remove(
            parser::parse(&tokenizer::tokenize(&content, "<", ">")),
            &content,
            &builder_map,
            &initialize_remove_strategy(Rc::clone(&content)),
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
