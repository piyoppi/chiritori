use lib_chiritori::chiritori::{
    ChiritoriConfiguration, RemovalMarkerConfiguration, TimeLimitedConfiguration,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, rc::Rc};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmChiritoriConfiguration {
    time_limited_configuration: WasmChiritoriTimeLimitedConfiguration,
    removal_marker_configuration: WasmChiritoriRemovalMarkerConfiguration,
}

impl From<WasmChiritoriConfiguration> for ChiritoriConfiguration {
    fn from(val: WasmChiritoriConfiguration) -> Self {
        ChiritoriConfiguration {
            time_limited_configuration: val.time_limited_configuration.into(),
            removal_marker_configuration: val.removal_marker_configuration.into(),
        }
    }
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmChiritoriTimeLimitedConfiguration {
    pub tag_name: String,
    pub time_offset: String,
    pub current: Option<String>,
}

impl From<WasmChiritoriTimeLimitedConfiguration> for TimeLimitedConfiguration {
    fn from(val: WasmChiritoriTimeLimitedConfiguration) -> Self {
        TimeLimitedConfiguration {
            tag_name: val.tag_name,
            time_offset: val.time_offset,
            current: val
                .current
                .and_then(|v| v.parse::<chrono::DateTime<chrono::Local>>().ok())
                .unwrap_or(chrono::Local::now()),
        }
    }
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmChiritoriRemovalMarkerConfiguration {
    pub tag_name: String,
    pub targets: HashSet<String>,
}

impl From<WasmChiritoriRemovalMarkerConfiguration> for RemovalMarkerConfiguration {
    fn from(val: WasmChiritoriRemovalMarkerConfiguration) -> Self {
        RemovalMarkerConfiguration {
            tag_name: val.tag_name,
            targets: val.targets,
        }
    }
}

#[wasm_bindgen]
pub fn list_all(
    content: String,
    delimiter_start: String,
    delimiter_end: String,
    config: WasmChiritoriConfiguration,
) -> String {
    let content = Rc::new(content);

    lib_chiritori::chiritori::list_all(content, (delimiter_start, delimiter_end), config.into())
}

#[wasm_bindgen]
pub fn clean(
    content: String,
    delimiter_start: String,
    delimiter_end: String,
    config: WasmChiritoriConfiguration,
) -> String {
    let content = Rc::new(content);

    lib_chiritori::chiritori::clean(content, (delimiter_start, delimiter_end), config.into())
}
