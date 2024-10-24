use super::{availability::MarkerAvailability, builder::MarkerBuilder};
use crate::parser::Element;
use std::ops::Range;

pub type RemoveStrategies = Vec<(Box<dyn MarkerAvailability>, Box<dyn MarkerBuilder>)>;

pub fn create(
    element: &Element,
    remove_strategy_map: &RemoveStrategies,
) -> Option<(Range<usize>, Option<Range<usize>>)> {
    remove_strategy_map
        .iter()
        .find(|(availability, _)| availability.is_available(element))
        .map(|(_, builder)| builder.build(element))
}
