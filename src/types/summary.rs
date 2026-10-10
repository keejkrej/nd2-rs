use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Summary information for an individual image acquisition channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryChannel {
    /// Zero-based channel index.
    pub index: usize,
    /// Channel display name, if defined in file metadata.
    pub name: Option<String>,
    /// Channel display color (e.g. hex or color string), if defined.
    pub color: Option<String>,
    /// Pixel data type name for this channel (e.g. "Unsigned16", "Float32").
    pub pixel_type: Option<String>,
}

/// Physical spatial calibration and unit metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryScaling {
    /// Physical pixel size along the X axis.
    pub x: Option<f64>,
    /// Physical pixel size along the Y axis.
    pub y: Option<f64>,
    /// Physical voxel step along the Z axis.
    pub z: Option<f64>,
    /// Calibration distance unit (e.g. "µm", "m").
    pub unit: Option<String>,
}

/// Lightweight, comprehensive overview of a microscopy dataset.
///
/// Provides key structural parameters including file version, dimension sizes,
/// logical frame counts, channel configurations, and physical scaling calibration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatasetSummary {
    /// Major format version (e.g. 2 or 3).
    pub version_major: u32,
    /// Minor format version (e.g. 0 or 1).
    pub version_minor: u32,
    /// Dimension map mapping dimension codes (e.g. "X", "Y", "C", "Z", "T", "P") to sizes.
    pub sizes: BTreeMap<String, usize>,
    /// Total number of logical frames (acquisitions) across all non-spatial dimensions.
    pub logical_frame_count: usize,
    /// Metadata for each channel present in the dataset.
    pub channels: Vec<SummaryChannel>,
    /// Primary or default pixel format name across the dataset.
    pub pixel_type: Option<String>,
    /// Spatial scaling calibration if recorded in the metadata.
    pub scaling: Option<SummaryScaling>,
}
