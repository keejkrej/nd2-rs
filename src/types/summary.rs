use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryChannel {
    pub index: usize,
    pub name: Option<String>,
    pub color: Option<String>,
    pub pixel_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryScaling {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatasetSummary {
    pub version_major: u32,
    pub version_minor: u32,
    pub sizes: BTreeMap<String, usize>,
    pub logical_frame_count: usize,
    pub channels: Vec<SummaryChannel>,
    pub pixel_type: Option<String>,
    pub scaling: Option<SummaryScaling>,
}
