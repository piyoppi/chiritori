pub mod removal_evaluator;
pub mod time_limited_evaluator;
pub mod marker;

use marker::factory::{create, RemoveStrategies};
use marker::RemoveMarker;
use removal_evaluator::RemovalEvaluator;
use crate::parser;
use crate::parser::ContentPart;
use std::collections::HashMap;

pub fn remove(
    content: Vec<ContentPart>,
    raw: &str,
    builder_map: &HashMap<&str, Box<dyn RemovalEvaluator>>,
    remove_strategy_map: &RemoveStrategies
) -> (String, Vec<RemoveMarker>) {
    let markers = remove_marker(&content, builder_map, remove_strategy_map);
    let mut new_content = raw.to_string();

    for marker in markers.iter().rev() {
        match marker {
            RemoveMarker::Block(range) => new_content.replace_range(range.byte_start..range.byte_end, ""),
            RemoveMarker::OpenStructure(v) => v.iter().for_each(|range| new_content.replace_range(range.byte_start..range.byte_end, ""))
        };
    }

    (new_content, markers)
}

pub fn get_removed_pos(markers: &[RemoveMarker]) -> Vec<usize> {
    markers
        .iter()
        .fold((vec![], 0), |(mut positions, mut removed_len), marker| {
            match marker {
                RemoveMarker::Block(range) => {
                    positions.push(range.byte_start - removed_len);
                    removed_len += range.byte_end - range.byte_start;
                },
                RemoveMarker::OpenStructure(v) => {
                    v.iter().for_each(|range| {
                        positions.push(range.byte_start - removed_len);
                        removed_len += range.byte_end - range.byte_start;
                    });
                }
            }
            (positions, removed_len)
        })
        .0
}

fn remove_marker(
    contents: &[ContentPart],
    evaluator_map: &HashMap<&str, Box<dyn RemovalEvaluator>>,
    remove_strategy_map: &RemoveStrategies
) -> Vec<RemoveMarker> {
    contents.iter().fold(vec![], |mut acc, c| {
        if let parser::ContentPart::Element(el) = c {
            let evaluator = evaluator_map.get(el.start_element.name);
            let marker = evaluator
                .and_then(|evaluator| {
                    match evaluator.is_removal(&el.start_element) {
                        true => create(el, remove_strategy_map),
                        false => None
                    }
                });

            if let Some(marker) = marker {
                acc.push(marker);
            } else {
                remove_marker(&el.children, evaluator_map, remove_strategy_map)
                    .into_iter()
                    .for_each(|m| acc.push(m));
            }
        }

        acc
    })
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use marker::{availability::{block_marker_availability::BlockMarkerAvailability, open_structure_marker_availability::OpenStructureMarkerAvailability}, builder::{block_marker_builder::BlockMarkerBuilder, open_structure_marker_builder::OpenStructureMarkerBuilder}, Range};
    use time_limited_evaluator::TimeLimitedEvaluator;

    use super::*;
    use crate::tokenizer;

    #[test]
    fn test_remove_marker() {
        //             0         1         2         3        4
        // byte_pos:   012345678901234567890123456789012-5678901234
        let content = "foo<tl to='2000-01-01 00:00:00'>あ</tl>fuga";
        let tokens = tokenizer::tokenize(content, "<", ">");
        let contents = parser::parse(&tokens);

        let mut builder_map: HashMap<&str, Box<dyn RemovalEvaluator>> = HashMap::new();
        builder_map.insert(
            "tl",
            Box::new(TimeLimitedEvaluator {
                current_time: chrono::Local::now(),
                time_offset: "+00:00".to_string(),
            }),
        );
        let remove_strategy_map: RemoveStrategies = vec![
            (
                Box::new(BlockMarkerAvailability::default()),
                Box::new(BlockMarkerBuilder::default())
            )
        ];

        assert_eq!(
            remove_marker(&contents, &builder_map, &remove_strategy_map),
            vec![RemoveMarker::Block(Range {
                byte_start: 3,
                byte_end: 40,
            }),]
        );

        let content = "foo<baz>a";
        let tokens = tokenizer::tokenize(content, "<", ">");
        let contents = parser::parse(&tokens);
        assert_eq!(remove_marker(&contents, &builder_map, &remove_strategy_map), vec![]);
    }

    #[test]
    fn test_remove() {
        let content = Rc::new("
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
".to_string());

        let mut builder_map: HashMap<&str, Box<dyn RemovalEvaluator>> = HashMap::new();
        builder_map.insert(
            "time-limited",
            Box::new(TimeLimitedEvaluator {
                current_time: chrono::Local::now(),
                time_offset: "+00:00".to_string(),
            }),
        );
        let remove_strategy_map: RemoveStrategies = vec![
            (
                Box::new(BlockMarkerAvailability::default()),
                Box::new(BlockMarkerBuilder::default())
            )
        ];

        assert_eq!(
            remove(
                parser::parse(&tokenizer::tokenize(&content, "<!--", "-->")),
                &content,
                &builder_map,
                &remove_strategy_map
            ),
            (
                "+<div>+    hoge+    +    foo+    bar+    baz+    +</div>+".replace('+', "\n"),
                vec![
                    RemoveMarker::Block(Range {
                        byte_start: 20,
                        byte_end: 117,
                    }),
                    RemoveMarker::Block(Range {
                        byte_start: 146,
                        byte_end: 243,
                    }),
                ]
            )
        );

        let content = "
hoge
<!-- time-limited to='2021-12-31 23:50:00' -->
<h1>Campaign 1</h1>
<!-- /time-limited -->
foo
<!-- time-limited to='2022-12-31 23:50:00' -->
<h1>Campaign 2</h1>
<!-- /time-limited -->
";

        assert_eq!(
            remove(
                parser::parse(&tokenizer::tokenize(content, "<!--", "-->")),
                content,
                &builder_map,
                &remove_strategy_map
            ),
            (
                "
hoge

foo

"
                .to_string(),
                vec![
                    RemoveMarker::Block(Range {
                        byte_start: 6,
                        byte_end: 95,
                    }),
                    RemoveMarker::Block(Range {
                        byte_start: 100,
                        byte_end: 189,
                    }),
                ]
            )
        );

        let content = Rc::new("
// --- start ---
/* time-limited to='2021-12-31 23:50:00' open-structure */
if (foo) {
    console.log('abc');
    console.log('def');
}
/* /time-limited */
// ---  end  ---
".to_string());

        let remove_strategy_map: RemoveStrategies = vec![
            (
                Box::new(OpenStructureMarkerAvailability::default()),
                Box::new(OpenStructureMarkerBuilder {
                    content: Rc::clone(&content)
                })
            ),
            (
                Box::new(BlockMarkerAvailability::default()),
                Box::new(BlockMarkerBuilder::default())
            )
        ];

        assert_eq!(
            remove(
                parser::parse(&tokenizer::tokenize(&content, "/*", "*/")),
                &content,
                &builder_map,
                &remove_strategy_map
            ),
            (
                "
// --- start ---

    console.log('abc');
    console.log('def');

// ---  end  ---
"
                .to_string(),
                vec![
                    RemoveMarker::OpenStructure(vec![
                        Range {
                            byte_start: 136,
                            byte_end: 157,
                        },
                        Range {
                            byte_start: 18,
                            byte_end: 87,
                        }
                    ])
                ]
            )
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
        let markers = vec![
            RemoveMarker::Block(Range {
                byte_start: 1,
                byte_end: 3,
            }),
            RemoveMarker::Block(Range {
                byte_start: 5,
                byte_end: 6,
            }),
            RemoveMarker::Block(Range {
                byte_start: 8,
                byte_end: 9,
            }),
        ];
        assert_eq!(get_removed_pos(&markers), vec![1, 3, 5]);
    }
}
