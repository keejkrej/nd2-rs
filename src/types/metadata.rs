use serde::{Deserialize, Serialize};

use super::PixelDataType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub contents: Option<Contents>,
    pub channels: Option<Vec<Channel>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contents {
    pub channel_count: u32,
    pub frame_count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Channel {
    pub channel: ChannelMeta,
    pub loops: Option<LoopIndices>,
    pub microscope: Microscope,
    pub volume: Volume,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelMeta {
    pub name: String,
    pub index: u32,
    pub color: Color,
    pub emission_lambda_nm: Option<f64>,
    pub excitation_lambda_nm: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn from_abgr_u32(val: u32) -> Self {
        Self {
            r: (val & 0xFF) as u8,
            g: ((val >> 8) & 0xFF) as u8,
            b: ((val >> 16) & 0xFF) as u8,
            a: ((val >> 24) & 0xFF) as u8,
        }
    }

    pub fn as_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoopIndices {
    pub ne_time_loop: Option<u32>,
    pub time_loop: Option<u32>,
    pub xy_pos_loop: Option<u32>,
    pub z_stack_loop: Option<u32>,
    pub custom_loop: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Microscope {
    pub objective_magnification: Option<f64>,
    pub objective_name: Option<String>,
    pub objective_numerical_aperture: Option<f64>,
    pub zoom_magnification: Option<f64>,
    pub immersion_refractive_index: Option<f64>,
    pub projective_magnification: Option<f64>,
    pub pinhole_diameter_um: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Volume {
    pub axes_calibrated: (bool, bool, bool),
    pub axes_calibration: (f64, f64, f64),
    pub axes_interpretation: (AxisInterpretation, AxisInterpretation, AxisInterpretation),
    pub bits_per_component_in_memory: u32,
    pub bits_per_component_significant: u32,
    pub camera_transformation_matrix: (f64, f64, f64, f64),
    pub component_count: u32,
    pub component_data_type: PixelDataType,
    pub voxel_count: (u32, u32, u32),
    pub component_maxima: Option<Vec<f64>>,
    pub component_minima: Option<Vec<f64>>,
    pub pixel_to_stage_transformation_matrix: Option<(f64, f64, f64, f64, f64, f64)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AxisInterpretation {
    Distance,
    Time,
}
