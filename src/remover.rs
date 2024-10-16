pub mod remove_marker_builder;
pub mod time_limited_remover;

use self::remove_marker_builder::RemoveMarkerBuilder;
use crate::parser;
use crate::parser::ContentPart;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct RemoveMarker {
    pub byte_start: usize,
    pub byte_end: usize,
}

fn remove_marker(
    contents: &[ContentPart],
    builder_map: &HashMap<&str, Box<dyn RemoveMarkerBuilder>>,
) -> Vec<RemoveMarker> {
    contents.iter().fold(vec![], |mut acc, c| {
        if let parser::ContentPart::Element(el) = c {
            let builder = builder_map.get(el.start_element.name);
            let marker = builder
                .and_then(|builder| {
                    builder.create_remove_marker(el.start_token, &el.start_element, el.end_token)
                });

            if let Some(marker) = marker {
                acc.push(marker);
            } else {
                remove_marker(&el.children, builder_map)
                    .into_iter()
                    .for_each(|m| acc.push(m));
            }
        }

        acc
    })
}

pub fn remove(
    content: Vec<ContentPart>,
    raw: &str,
    builder_map: &HashMap<&str, Box<dyn RemoveMarkerBuilder>>,
) -> (String, Vec<RemoveMarker>) {
    let markers = remove_marker(&content, builder_map);
    let mut new_content = raw.to_string();

    for marker in markers.iter().rev() {
        new_content.replace_range(marker.byte_start..marker.byte_end, "");
    }

    (new_content, markers)
}

pub fn get_removed_pos(markers: &[RemoveMarker]) -> Vec<usize> {
    markers
        .iter()
        .fold((vec![], 0), |mut acc, marker| {
            acc.0.push(marker.byte_start - acc.1);
            (acc.0, acc.1 + marker.byte_end - marker.byte_start)
        })
        .0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer;

    #[test]
    fn test_remove_marker() {
        //             0         1         2         3        4
        // byte_pos:   012345678901234567890123456789012-5678901234
        let content = "foo<tl to='2000-01-01 00:00:00'>あ</tl>fuga";
        let tokens = tokenizer::tokenize(content, "<", ">");
        let contents = parser::parse(&tokens);

        let mut builder_map: HashMap<&str, Box<dyn RemoveMarkerBuilder>> = HashMap::new();
        builder_map.insert(
            "tl",
            Box::new(time_limited_remover::TimeLimitedRemover {
                current_time: chrono::Local::now(),
                time_offset: "+00:00".to_string(),
            }),
        );
        assert_eq!(
            remove_marker(&contents, &builder_map),
            vec![RemoveMarker {
                byte_start: 3,
                byte_end: 40,
            },]
        );

        let content = "foo<baz>a";
        let tokens = tokenizer::tokenize(content, "<", ">");
        let contents = parser::parse(&tokens);
        assert_eq!(remove_marker(&contents, &builder_map), vec![]);
    }

    #[test]
    fn test_remove() {
        let content = "
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
";

        let mut builder_map: HashMap<&str, Box<dyn RemoveMarkerBuilder>> = HashMap::new();
        builder_map.insert(
            "time-limited",
            Box::new(time_limited_remover::TimeLimitedRemover {
                current_time: chrono::Local::now(),
                time_offset: "+00:00".to_string(),
            }),
        );
        assert_eq!(
            remove(
                parser::parse(&tokenizer::tokenize(content, "<!--", "-->")),
                content,
                &builder_map
            ),
            (
                "+<div>+    hoge+    +    foo+    bar+    baz+    +</div>+".replace('+', "\n"),
                vec![
                    RemoveMarker {
                        byte_start: 20,
                        byte_end: 117,
                    },
                    RemoveMarker {
                        byte_start: 146,
                        byte_end: 243,
                    },
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
                &builder_map
            ),
            (
                "
hoge

foo

"
                .to_string(),
                vec![
                    RemoveMarker {
                        byte_start: 6,
                        byte_end: 95,
                    },
                    RemoveMarker {
                        byte_start: 100,
                        byte_end: 189,
                    },
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
            RemoveMarker {
                byte_start: 1,
                byte_end: 3,
            },
            RemoveMarker {
                byte_start: 5,
                byte_end: 6,
            },
            RemoveMarker {
                byte_start: 8,
                byte_end: 9,
            },
        ];
        assert_eq!(get_removed_pos(&markers), vec![1, 3, 5]);
    }
}
