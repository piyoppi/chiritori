use std::{collections::HashMap, rc::Rc};
use crate::{code::{formatter::{self, Formatter, StructureFormatter}, remover::{self, marker::{availability::{block_marker_availability::BlockMarkerAvailability, open_structure_marker_availability::OpenStructureMarkerAvailability}, builder::{block_marker_builder::BlockMarkerBuilder, open_structure_marker_builder::OpenStructureMarkerBuilder}, factory::RemoveStrategies}}}, parser, tokenizer};

pub struct ChiritoriConfiguration {
    pub delimiter_start: String,
    pub delimiter_end: String,
    pub time_limited_configuration: TimeLimitedConfiguration
}

impl Default for ChiritoriConfiguration {
    fn default() -> Self {
        Self {
            delimiter_start: String::from("<!--"),
            delimiter_end: String::from("-->"),
            time_limited_configuration: TimeLimitedConfiguration::default() 
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
            current: chrono::Local::now()
        }
    }
}

pub fn clean(content: Rc<String>, config: ChiritoriConfiguration) -> String {
    let tokens = tokenizer::tokenize(
        &content,
        &config.delimiter_start,
        &config.delimiter_end,
    );

    let parsed = parser::parse(&tokens);
    let mut builder_map: HashMap<
        &str,
        Box<dyn remover::removal_evaluator::RemovalEvaluator>,
    > = HashMap::new();
    builder_map.insert(
        &config.time_limited_configuration.tag_name,
        Box::new(remover::time_limited_evaluator::TimeLimitedEvaluator {
            current_time: config.time_limited_configuration.current,
            time_offset: config.time_limited_configuration.time_offset,
        }),
    );

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
    let structure_formatters: Vec<Box<dyn StructureFormatter>> = vec! [

    ];

    formatter::format(&removed, &removed_pos, &formatter, &structure_formatters)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    fn create_test_config() -> ChiritoriConfiguration {
        ChiritoriConfiguration {
            delimiter_start: String::from("<!--"),
            delimiter_end: String::from("-->"),
            time_limited_configuration: TimeLimitedConfiguration {
                tag_name: String::from("time-limited"),
                current: Local::now(),
                time_offset: String::from("+00:00"),
            },
        }
    }

    #[test]
    fn test_clean_removes_time_limited_code() {
        let content = String::from(r#"
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
</html>"#);

         let expected = String::from(r#"
<!DOCTYPE html>
<html>
  <body>
    <h1>Hello World</h1>
    <p>This is a sample page with some code.</p>
    <!-- time-limited to="2999-12-31 23:59:59" -->
      Campaign!
    <!-- /time-limited -->
  </body>
</html>"#);

        let config = create_test_config();
        let result = clean(content.into(), config);

        assert_eq!(result, expected);
    }
}
