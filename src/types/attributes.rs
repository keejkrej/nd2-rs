use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attributes {
    pub bits_per_component_in_memory: u32,
    pub bits_per_component_significant: u32,
    pub component_count: u32,
    pub height_px: u32,
    pub pixel_data_type: PixelDataType,
    pub sequence_count: u32,
    pub width_bytes: Option<u32>,
    pub width_px: Option<u32>,
    pub compression_level: Option<f64>,
    pub compression_type: Option<CompressionType>,
    pub tile_height_px: Option<u32>,
    pub tile_width_px: Option<u32>,
    pub channel_count: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PixelDataType {
    Float,
    Unsigned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionType {
    Lossless,
    Lossy,
    None,
}
