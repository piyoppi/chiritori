use crate::{
    code::{
        formatter::{self, BlockFormatter, Formatter},
        remover::{
            self,
            marker::{
                availability::{
                    range_marker_availability::RangeMarkerAvailability,
                    unwrap_block_marker_availability::UnwrapBlockMarkerAvailability,
                },
                builder::{
                    range_marker_builder::RangeMarkerBuilder,
                    unwrap_block_marker_builder::UnwrapBlockMarkerBuilder,
                },
                factory::RemoveStrategies,
            },
            removal_evaluator::RemovalEvaluator,
        },
    },
    parser, tokenizer,
};
use std::{collections::HashMap, rc::Rc};

pub struct ChiritoriConfiguration {
    pub delimiter_start: String,
    pub delimiter_end: String,
    pub time_limited_configuration: TimeLimitedConfiguration,
}

impl Default for ChiritoriConfiguration {
    fn default() -> Self {
        Self {
            delimiter_start: String::from("<!--"),
            delimiter_end: String::from("-->"),
            time_limited_configuration: TimeLimitedConfiguration::default(),
        }
    }
}

pub struct TimeLimitedConfiguration {
    pub tag_name: String,
    pub time_offset: String,
    pub current: chrono::DateTime<chrono::Local>,
}

impl Default for TimeLimitedConfiguration {
    fn default() -> Self {
        Self {
            tag_name: String::from("time-limited"),
            time_offset: String::from("+00:00"),
            current: chrono::Local::now(),
        }
    }
}

pub fn clean(content: Rc<String>, config: ChiritoriConfiguration) -> String {
    let tokens = tokenizer::tokenize(&content, &config.delimiter_start, &config.delimiter_end);

    let parsed = parser::parse(&tokens);
    let mut builder_map: HashMap<&str, Box<dyn RemovalEvaluator>> = HashMap::new();

    builder_map.insert(
        &config.time_limited_configuration.tag_name,
        Box::new(remover::time_limited_evaluator::TimeLimitedEvaluator {
            current_time: config.time_limited_configuration.current,
            time_offset: config.time_limited_configuration.time_offset,
        }),
    );

    let remove_strategy_map: RemoveStrategies = vec![
        (
            Box::new(UnwrapBlockMarkerAvailability::new("unwrap-block")),
            Box::new(UnwrapBlockMarkerBuilder {
                content: Rc::clone(&content),
            }),
        ),
        (
            Box::new(RangeMarkerAvailability::default()),
            Box::new(RangeMarkerBuilder::default()),
        ),
    ];

    let (removed, markers) = remover::remove(parsed, &content, &builder_map, &remove_strategy_map);

    let removed_pos = remover::get_removed_pos(&markers);
    let formatter: Vec<Box<dyn Formatter>> = vec![
        Box::new(formatter::indent_remover::IndentRemover {}),
        Box::new(formatter::empty_line_remover::EmptyLineRemover {}),
        Box::new(formatter::prev_line_break_remover::PrevLineBreakRemover {}),
        Box::new(formatter::next_line_break_remover::NextLineBreakRemover {}),
    ];
    let structure_formatters: Vec<Box<dyn BlockFormatter>> = vec![Box::new(
        formatter::block_indent_remover::BlockIndentRemover {},
    )];

    formatter::format(&removed, &removed_pos, &formatter, &structure_formatters)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    fn create_test_config(delimiter_start: &str, delimiter_end: &str) -> ChiritoriConfiguration {
        ChiritoriConfiguration {
            delimiter_start: String::from(delimiter_start),
            delimiter_end: String::from(delimiter_end),
            time_limited_configuration: TimeLimitedConfiguration {
                tag_name: String::from("time-limited"),
                current: Local::now(),
                time_offset: String::from("+00:00"),
            },
        }
    }

    #[test]
    fn test_clean_removes_time_limited_code() {
        let content = String::from(
            r#"
<!DOCTYPE html>
<html>
  <body>
    <h1>Hello World</h1>
    <p>This is a sample page with some code.</p>
    <!-- time-limited to="2019-12-31 23:59:59" -->
      <p>This content is only available until the end of the year.</p>
      <p>After that, it will be removed from the page.</p>
    <!-- /time-limited -->
    <!-- time-limited to="2999-12-31 23:59:59" -->
      Campaign!
      <!-- time-limited to="2001-12-31 23:59:59" -->
        40% off! Only until the 2001/12/31!
      <!-- /time-limited -->
    <!-- /time-limited -->
    <!-- time-limited to="2001-12-31 23:59:59" -->
      <!-- time-limited to="2999-12-31 23:59:59" -->
        Campaign!
      <!-- /time-limited -->
      until the 2001/12/31!
    <!-- /time-limited -->
  </body>
</html>"#,
        );

        let expected = String::from(
            r#"
<!DOCTYPE html>
<html>
  <body>
    <h1>Hello World</h1>
    <p>This is a sample page with some code.</p>
    <!-- time-limited to="2999-12-31 23:59:59" -->
      Campaign!
    <!-- /time-limited -->
  </body>
</html>"#,
        );

        let config = create_test_config("<!--", "-->");
        let result = clean(content.into(), config);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_clean_removes_time_limited_code_with_open_structure() {
        let content = String::from(
            r#"
isReleased = async fetchFeature("https://example.test/features/awesome-feature")

/* <time-limited to="2021-01-01 00:00:00" unwrap-block> */
if (isReleased) {
  console.log("Released!")

  /* <time-limited to="2021-01-01 00:00:00"> */
  console.log("Temporary code until 2021/01/01")
  /* </time-limited> */

  /* <time-limited to="9999-01-01 00:00:00"> */
  console.log("Temporary code until 9999/01/01")
  /* </time-limited> */
}
/* </time-limited> */
"#,
        );
        let expected = String::from(
            r#"
isReleased = async fetchFeature("https://example.test/features/awesome-feature")

console.log("Released!")

/* <time-limited to="9999-01-01 00:00:00"> */
console.log("Temporary code until 9999/01/01")
/* </time-limited> */
"#,
        );

        let config = create_test_config("/* <", "> */");
        let result = clean(content.into(), config);

        assert_eq!(result, expected);
    }
}
