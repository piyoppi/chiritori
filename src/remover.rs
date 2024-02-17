pub mod remove_marker_builder;
pub mod time_limited_remover;

use std::collections::HashMap;
use crate::parser::{Content, ContentPart};
use crate::parser;
use crate::element_parser::Element;
use self::remove_marker_builder::RemoveMarkerBuilder;
use crate::tokenizer::Token;

#[derive(Debug, PartialEq)]
pub struct RemoveMarker {
    pub byte_start: usize,
    pub byte_end: usize,
}

fn remove_marker(contents: Vec<ContentPart>, builder_map: &HashMap<&str, Box<dyn RemoveMarkerBuilder>>) -> Vec<RemoveMarker> {
    contents.iter()
        .fold((vec![], 0, None, None), |mut acc: (Vec<RemoveMarker>, usize, Option<&Token>, Option<&Element>), c: &ContentPart| {
            if !matches!(c.kind, parser::ContentKind::Element(_)) {
                return acc;
            }

            let el = match &c.kind {
                parser::ContentKind::Element(e) => e,
                _ => panic!("Should not happen"),
            };

            if el.name.starts_with("/") && acc.1 > 0 {
                let builder = builder_map.get(el.name.trim_start_matches('/'));

                if builder.is_some() {
                    acc.1 = acc.1 - 1;
                    if acc.1 == 0 {
                        let builder = builder.unwrap();
                        let marker = builder.create_remove_marker(acc.2.unwrap(), acc.3.unwrap(), &c.token);
                        if marker.is_some() {
                            acc.0.push(marker.unwrap());
                        }
                    }
                }
            } else {
                let builder = builder_map.contains_key(el.name.as_str());
                if builder {
                    if acc.1 == 0 {
                        acc.2 = Some(&c.token);
                        acc.3 = Some(&el);
                    }
                    acc.1 = acc.1 + 1;
                }
            }

            acc
        })
        .0
}

#[test]
fn test_remove_marker() {
    let content = "
<div>
    <!-- time-limited to='2021-12-31 23:50:00' -->
    <h1>Hello, World!</h1>
    <!-- /time-limited -->
</div>";

    let contents = parser::parse(content, "<!--", "-->");

    let mut builder_map: HashMap<&str, Box<dyn RemoveMarkerBuilder>> = HashMap::new();
    builder_map.insert("time-limited", Box::new(time_limited_remover::TimeLimitedRemover {
        current_time: chrono::Local::now(),
        time_offset: "+00:00".to_string(),
    }));
    assert_eq!(
        remove_marker(contents.parts, &builder_map),
        vec![
            RemoveMarker {
                byte_start: 11,
                byte_end: 111,
            },
        ]
    );
}

pub fn remove(content: Content, builder_map: &HashMap<&str, Box<dyn RemoveMarkerBuilder>>) -> (String, Vec<RemoveMarker>) {
    let markers = remove_marker(content.parts, builder_map);
    let mut new_content = content.raw.to_string();

    for marker in markers.iter().rev() {
        new_content.replace_range(marker.byte_start..marker.byte_end, "");
    }

    (new_content, markers)
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
    builder_map.insert("time-limited", Box::new(time_limited_remover::TimeLimitedRemover {
        current_time: chrono::Local::now(),
        time_offset: "+00:00".to_string(),
    }));
    assert_eq!(
        remove(parser::parse(content, "<!--", "-->"), &builder_map),
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
        remove(parser::parse(content, "<!--", "-->"), &builder_map),
        (
            "
hoge

foo

".to_string(),
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

pub fn get_removed_pos(markers: &Vec<RemoveMarker>) -> Vec<usize> {
    markers.iter().fold((vec![], 0), |mut acc, marker| {
        acc.0.push(marker.byte_start - acc.1);
        (acc.0, acc.1 + marker.byte_end - marker.byte_start)
    }).0
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
    assert_eq!(
        get_removed_pos(&markers),
        vec![1, 3, 5]
    );
}
