use crate::error::{Nd2Error, Result};
use crate::parse::ClxValue;
use crate::types::{
    CustomLoop, ExpLoop, NETimeLoop, NETimeLoopParams, Period, Position,
    StagePosition, TimeLoop, TimeLoopParams, XYPosLoop, XYPosLoopParams, ZStackLoop,
    ZStackLoopParams,
};

pub fn parse_experiment(clx: ClxValue) -> Result<Vec<ExpLoop>> {
    let obj = clx
        .as_object()
        .ok_or_else(|| Nd2Error::MetadataParse("Expected object for experiment".to_string()))?;

    let mut loops = Vec::new();

    // Parse ppNextLevelEx array
    if let Some(next_level) = obj.get("ppNextLevelEx") {
        if let ClxValue::Array(arr) = next_level {
            for item in arr {
                if let Some(loop_obj) = item.as_object() {
                    if let Some(exp_loop) = parse_single_loop(loop_obj)? {
                        loops.push(exp_loop);
                    }
                }
            }
        }
    }

    Ok(loops)
}

fn parse_single_loop(obj: &std::collections::HashMap<String, ClxValue>) -> Result<Option<ExpLoop>> {
    let get_u32 = |key: &str| -> Option<u32> {
        obj.get(key).and_then(|v| v.as_u64()).map(|v| v as u32)
    };

    let get_f64 = |key: &str| -> Option<f64> { obj.get(key).and_then(|v| v.as_f64()) };

    let get_bool = |key: &str| -> Option<bool> { obj.get(key).and_then(|v| v.as_bool()) };

    let loop_type = get_u32("uiLoopType");
    let count = get_u32("uiCount").unwrap_or(0);
    let nesting_level = get_u32("uiNestingLevel").unwrap_or(0);

    match loop_type {
        Some(1) => {
            // TimeLoop
            let params = TimeLoopParams {
                start_ms: get_f64("dPeriod").unwrap_or(0.0) * 1000.0,
                period_ms: get_f64("dAvgPeriodDiff").unwrap_or(0.0) * 1000.0,
                duration_ms: get_f64("dDuration").unwrap_or(0.0) * 1000.0,
                period_diff: None, // TODO: parse if present
            };
            Ok(Some(ExpLoop::TimeLoop(TimeLoop {
                count,
                nesting_level,
                parameters: params,
            })))
        }
        Some(4) => {
            // ZStackLoop
            let params = ZStackLoopParams {
                home_index: get_u32("uiHomeIndex").map(|v| v as i32).unwrap_or(0),
                step_um: get_f64("dZStep").unwrap_or(0.0),
                bottom_to_top: get_bool("bBottomToTop").unwrap_or(false),
                device_name: obj
                    .get("pPeriod")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            };
            Ok(Some(ExpLoop::ZStackLoop(ZStackLoop {
                count,
                nesting_level,
                parameters: params,
            })))
        }
        Some(2) => {
            // XYPosLoop
            let is_setting_z = get_bool("bIsSettingZ").unwrap_or(false);
            let mut points = Vec::new();

            if let Some(pos_list) = obj.get("pPeriod") {
                if let ClxValue::Array(arr) = pos_list {
                    for pos_item in arr {
                        if let Some(pos_obj) = pos_item.as_object() {
                            let x = pos_obj
                                .get("dPosX")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0);
                            let y = pos_obj
                                .get("dPosY")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0);
                            let z = pos_obj
                                .get("dPosZ")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0);
                            let pfs_offset =
                                pos_obj.get("dPFSOffset").and_then(|v| v.as_f64());
                            let name = pos_obj
                                .get("wszName")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());

                            points.push(Position {
                                stage_position_um: StagePosition { x, y, z },
                                pfs_offset,
                                name,
                            });
                        }
                    }
                }
            }

            let params = XYPosLoopParams {
                is_setting_z,
                points,
            };
            Ok(Some(ExpLoop::XYPosLoop(XYPosLoop {
                count,
                nesting_level,
                parameters: params,
            })))
        }
        Some(8) => {
            // NETimeLoop
            let mut periods = Vec::new();

            if let Some(period_list) = obj.get("pPeriod") {
                if let ClxValue::Array(arr) = period_list {
                    for period_item in arr {
                        if let Some(period_obj) = period_item.as_object() {
                            let count = period_obj
                                .get("uiCount")
                                .and_then(|v| v.as_u64())
                                .map(|v| v as u32)
                                .unwrap_or(0);
                            let start_ms = period_obj
                                .get("dStart")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0)
                                * 1000.0;
                            let period_ms = period_obj
                                .get("dAvgPeriodDiff")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0)
                                * 1000.0;
                            let duration_ms = period_obj
                                .get("dDuration")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0)
                                * 1000.0;

                            periods.push(Period {
                                count,
                                start_ms,
                                period_ms,
                                duration_ms,
                                period_diff: None,
                            });
                        }
                    }
                }
            }

            let params = NETimeLoopParams { periods };
            Ok(Some(ExpLoop::NETimeLoop(NETimeLoop {
                count,
                nesting_level,
                parameters: params,
            })))
        }
        Some(7) => {
            // CustomLoop
            Ok(Some(ExpLoop::CustomLoop(CustomLoop {
                count,
                nesting_level,
            })))
        }
        _ => Ok(None),
    }
}
