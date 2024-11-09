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
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub struct ChiritoriConfiguration {
    pub delimiter_start: String,
    pub delimiter_end: String,
    pub time_limited_configuration: TimeLimitedConfiguration,
    pub marker_tag_configuration: MarkerTagConfiguration,
}

pub struct TimeLimitedConfiguration {
    pub tag_name: String,
    pub time_offset: String,
    pub current: chrono::DateTime<chrono::Local>,
}

pub struct MarkerTagConfiguration {
    pub tag_name: String,
    pub marker_removal_tags: HashSet<String>,
}

pub fn clean(content: Rc<String>, config: ChiritoriConfiguration) -> String {
    let tokens = tokenizer::tokenize(&content, &config.delimiter_start, &config.delimiter_end);

    let parsed = parser::parse(&tokens);
    let mut builder_map: HashMap<&str, Box<dyn RemovalEvaluator>> = HashMap::new();

    builder_map.insert(
        &config.time_limited_configuration.tag_name,
        Box::new(
            remover::removal_evaluator::time_limited_evaluator::TimeLimitedEvaluator {
                current_time: config.time_limited_configuration.current,
                time_offset: config.time_limited_configuration.time_offset,
            },
        ),
    );

    builder_map.insert(
        &config.marker_tag_configuration.tag_name,
        Box::new(
            remover::removal_evaluator::marker_evaluator::MarkerEvaluator {
                marker_removal_names: config.marker_tag_configuration.marker_removal_tags,
            },
        ),
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
    use rstest::rstest;
    use std::ffi::OsString;
    use std::io::prelude::*;
    use std::{fs::File, path::PathBuf};

    fn create_test_config(delimiter_start: &str, delimiter_end: &str) -> ChiritoriConfiguration {
        ChiritoriConfiguration {
            delimiter_start: String::from(delimiter_start),
            delimiter_end: String::from(delimiter_end),
            time_limited_configuration: TimeLimitedConfiguration {
                tag_name: String::from("time-limited"),
                current: Local::now(),
                time_offset: String::from("+00:00"),
            },
            marker_tag_configuration: MarkerTagConfiguration {
                tag_name: String::from("marker"),
                marker_removal_tags: HashSet::from([String::from("feature1")]),
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

/* <marker name="feature1"> */
console.log("Temporary code while feature1 is not released")
/* </marker> */

/* <marker name="feature2"> */
console.log("Temporary code while feature2 is not released")
/* </marker> */
"#,
        );
        let expected = String::from(
            r#"
isReleased = async fetchFeature("https://example.test/features/awesome-feature")

console.log("Released!")

/* <time-limited to="9999-01-01 00:00:00"> */
console.log("Temporary code until 9999/01/01")
/* </time-limited> */

/* <marker name="feature2"> */
console.log("Temporary code while feature2 is not released")
/* </marker> */
"#,
        );

        let config = create_test_config("/* <", "> */");
        let result = clean(content.into(), config);

        assert_eq!(result, expected);
    }

    #[rstest]
    fn should_clean_with_js(
        #[files("src/integration-test-fixtures/*.input.js")] path_input: PathBuf,
    ) {
        let mut path_expected = path_input.clone();
        let mut filename_expected = OsString::from(path_input.file_name().unwrap());
        filename_expected.push(".expected");
        path_expected.set_file_name(filename_expected);

        let mut f_input = File::open(path_input).unwrap();
        let mut f_expected = File::open(path_expected).unwrap();
        let mut input_content = String::new();
        let mut expected_content = String::new();

        f_input
            .read_to_string(&mut input_content)
            .expect("Failed to load input a content file");
        f_expected
            .read_to_string(&mut expected_content)
            .expect("Failed to load an expected content file");

        let config = create_test_config("/* <", "> */");
        let result = clean(input_content.into(), config);

        assert_eq!(result, expected_content);
    }
}
