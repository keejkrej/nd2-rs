use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum LoopType {
    Unknown = 0,
    TimeLoop = 1,
    XYPosLoop = 2,
    XYDiscrLoop = 3,
    ZStackLoop = 4,
    PolarLoop = 5,
    SpectLoop = 6,
    CustomLoop = 7,
    NETimeLoop = 8,
    ManTimeLoop = 9,
    ZStackLoopAccurate = 10,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExpLoop {
    TimeLoop(TimeLoop),
    NETimeLoop(NETimeLoop),
    XYPosLoop(XYPosLoop),
    ZStackLoop(ZStackLoop),
    CustomLoop(CustomLoop),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeLoop {
    pub count: u32,
    pub nesting_level: u32,
    pub parameters: TimeLoopParams,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeLoopParams {
    pub start_ms: f64,
    pub period_ms: f64,
    pub duration_ms: f64,
    pub period_diff: Option<PeriodDiff>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeriodDiff {
    pub avg: f64,
    pub max: f64,
    pub min: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZStackLoop {
    pub count: u32,
    pub nesting_level: u32,
    pub parameters: ZStackLoopParams,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZStackLoopParams {
    pub home_index: i32,
    pub step_um: f64,
    pub bottom_to_top: bool,
    pub device_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XYPosLoop {
    pub count: u32,
    pub nesting_level: u32,
    pub parameters: XYPosLoopParams,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XYPosLoopParams {
    pub is_setting_z: bool,
    pub points: Vec<Position>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub stage_position_um: StagePosition,
    pub pfs_offset: Option<f64>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StagePosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NETimeLoop {
    pub count: u32,
    pub nesting_level: u32,
    pub parameters: NETimeLoopParams,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NETimeLoopParams {
    pub periods: Vec<Period>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Period {
    pub count: u32,
    pub start_ms: f64,
    pub period_ms: f64,
    pub duration_ms: f64,
    pub period_diff: Option<PeriodDiff>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomLoop {
    pub count: u32,
    pub nesting_level: u32,
}
