use crate::error::Result;
use crate::parse::ClxValue;
use crate::types::{
    CustomLoop, ExpLoop, NETimeLoop, NETimeLoopParams, Period, Position,
    StagePosition, TimeLoop, TimeLoopParams, XYPosLoop, XYPosLoopParams, ZStackLoop,
    ZStackLoopParams,
};

pub fn parse_experiment(clx: ClxValue) -> Result<Vec<ExpLoop>> {
    parse_experiment_inner(unwrap_single_item(clx), 0, Vec::new())
}

fn unwrap_single_item(mut v: ClxValue) -> ClxValue {
    loop {
        let next = match &v {
            ClxValue::Array(arr) if arr.len() == 1 => Some(arr[0].clone()),
            ClxValue::Object(map) if map.len() == 1 => map
                .get("")
                .or_else(|| map.get("i0000000000"))
                .or_else(|| map.get("SLxExperiment"))
                .cloned(),
            _ => None,
        };
        if let Some(n) = next {
            v = n;
        } else {
            break;
        }
    }
    v
}

fn parse_experiment_inner(
    clx: ClxValue,
    _level: u32,
    mut dest: Vec<ExpLoop>,
) -> Result<Vec<ExpLoop>> {
    let clx = unwrap_single_item(clx);
    let Some(obj) = clx.as_object() else {
        // Some files include non-object values under ppNextLevelEx; ignore them.
        return Ok(dest);
    };

    if let Some(exp_loop) = parse_single_loop(obj)? {
        if exp_loop.count() > 0 {
            dest.push(exp_loop);
        }
    }

    // Parse ppNextLevelEx - can be Array (v2) or Object/dict (v3)
    if let Some(next_level) = obj.get("ppNextLevelEx") {
        let items: Vec<&ClxValue> = match next_level {
            ClxValue::Array(arr) => arr.iter().collect(),
            ClxValue::Object(map) => {
                // Some files decode ppNextLevelEx directly as a single loop object
                // (containing eType/uLoopPars), rather than an indexed dict.
                if map.contains_key("eType") || map.contains_key("uiLoopType") {
                    vec![next_level]
                } else {
                    let mut keys: Vec<_> = map.keys().collect();
                    keys.sort();
                    keys.into_iter().filter_map(|k| map.get(k)).collect()
                }
            }
            _ => Vec::new(),
        };

        for item in items {
            let inner = unwrap_single_item(item.clone());
            dest = parse_experiment_inner(inner, _level + 1, dest)?;
        }
    }

    Ok(dest)
}

fn value_as_u32(v: &ClxValue) -> Option<u32> {
    v.as_u64()
        .map(|u| u as u32)
        .or_else(|| v.as_i64().and_then(|i| (i >= 0).then_some(i as u32)))
}

fn value_as_f64(v: &ClxValue) -> Option<f64> {
    v.as_f64()
        .or_else(|| v.as_u64().map(|u| u as f64))
        .or_else(|| v.as_i64().map(|i| i as f64))
}

fn value_as_bool(v: &ClxValue) -> Option<bool> {
    v.as_bool()
        .or_else(|| v.as_u64().map(|u| u != 0))
        .or_else(|| v.as_i64().map(|i| i != 0))
}

fn map_get_u32(map: &std::collections::HashMap<String, ClxValue>, key: &str) -> Option<u32> {
    map.get(key).and_then(value_as_u32)
}

fn map_get_f64(map: &std::collections::HashMap<String, ClxValue>, key: &str) -> Option<f64> {
    map.get(key).and_then(value_as_f64)
}

fn map_get_bool(map: &std::collections::HashMap<String, ClxValue>, key: &str) -> Option<bool> {
    map.get(key).and_then(value_as_bool)
}

fn parse_single_loop(obj: &std::collections::HashMap<String, ClxValue>) -> Result<Option<ExpLoop>> {
    let loop_type = map_get_u32(obj, "uiLoopType").or_else(|| map_get_u32(obj, "eType"));
    let nesting_level = map_get_u32(obj, "uiNestingLevel").unwrap_or(0);

    // nd2-py parses loop params from exp["uLoopPars"].
    let mut loop_pars = obj.get("uLoopPars").and_then(|u| u.as_object());
    if let Some(pars) = loop_pars {
        // nd2-py: if list(loop_params) == ["i0000000000"], unwrap one level.
        if pars.len() == 1 {
            if let Some(inner) = pars.get("i0000000000").and_then(|v| v.as_object()) {
                loop_pars = Some(inner);
            }
        }
    }
    let params = loop_pars.unwrap_or(obj);
    let loop_count = map_get_u32(params, "uiCount")
        .or_else(|| map_get_u32(obj, "uiCount"))
        .unwrap_or(0);

    match loop_type {
        Some(1) => {
            // TimeLoop
            if loop_count == 0 {
                return Ok(None);
            }
            let params = TimeLoopParams {
                start_ms: map_get_f64(params, "dStart").unwrap_or(0.0),
                period_ms: map_get_f64(params, "dPeriod").unwrap_or(0.0),
                duration_ms: map_get_f64(params, "dDuration").unwrap_or(0.0),
                period_diff: None,
            };
            Ok(Some(ExpLoop::TimeLoop(TimeLoop {
                count: loop_count,
                nesting_level,
                parameters: params,
            })))
        }
        Some(4) => {
            // ZStackLoop
            if loop_count == 0 {
                return Ok(None);
            }
            let params = ZStackLoopParams {
                home_index: map_get_u32(params, "uiHomeIndex")
                    .or_else(|| map_get_f64(params, "dZHome").map(|v| v as u32))
                    .map(|v| v as i32)
                    .unwrap_or(0),
                step_um: map_get_f64(params, "dZStep").unwrap_or(0.0),
                bottom_to_top: map_get_bool(params, "bBottomToTop")
                    .or_else(|| map_get_u32(params, "iType").map(|v| v < 4))
                    .unwrap_or(false),
                device_name: params
                    .get("wsZDevice")
                    .or_else(|| params.get("pPeriod"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            };
            Ok(Some(ExpLoop::ZStackLoop(ZStackLoop {
                count: loop_count,
                nesting_level,
                parameters: params,
            })))
        }
        Some(2) => {
            // XYPosLoop
            let is_setting_z = map_get_bool(params, "bUseZ")
                .or_else(|| map_get_bool(params, "bIsSettingZ"))
                .or_else(|| map_get_bool(obj, "bIsSettingZ"))
                .unwrap_or(false);
            let rel_xy = map_get_bool(params, "bRelativeXY").unwrap_or(false);
            let ref_x = if rel_xy {
                map_get_f64(params, "dReferenceX").unwrap_or(0.0)
            } else {
                0.0
            };
            let ref_y = if rel_xy {
                map_get_f64(params, "dReferenceY").unwrap_or(0.0)
            } else {
                0.0
            };

            let valid: Vec<bool> = obj
                .get("pItemValid")
                .and_then(|v| match v {
                    ClxValue::Object(m) => {
                        let mut keys: Vec<_> = m.keys().collect();
                        keys.sort();
                        Some(
                            keys.into_iter()
                                .filter_map(|k| m.get(k))
                                .map(|x| value_as_bool(x).unwrap_or(true))
                                .collect(),
                        )
                    }
                    ClxValue::Array(arr) => Some(
                        arr.iter()
                            .map(|x| value_as_bool(x).unwrap_or(true))
                            .collect(),
                    ),
                    ClxValue::ByteArray(bytes) => Some(bytes.iter().map(|b| *b != 0).collect()),
                    _ => None,
                })
                .unwrap_or_default();

            let pos_list = params
                .get("Points")
                .or_else(|| params.get("pPeriod"))
                .or_else(|| obj.get("pPeriod"));
            let mut points = Vec::new();
            if let Some(pos_list) = pos_list {
                let pos_items: Vec<&ClxValue> = match pos_list {
                    ClxValue::Array(arr) => arr.iter().collect(),
                    ClxValue::Object(map) => {
                        let mut keys: Vec<_> = map.keys().collect();
                        keys.sort();
                        keys.into_iter().filter_map(|k| map.get(k)).collect()
                    }
                    _ => Vec::new(),
                };

                for (i, pos_item) in pos_items.into_iter().enumerate() {
                    if !valid.is_empty() && (i >= valid.len() || !valid[i]) {
                        continue;
                    }
                    let pos_item = unwrap_single_item(pos_item.clone());
                    if let Some(pos_obj) = pos_item.as_object() {
                        let x = ref_x + map_get_f64(pos_obj, "dPosX").unwrap_or(0.0);
                        let y = ref_y + map_get_f64(pos_obj, "dPosY").unwrap_or(0.0);
                        let z = if is_setting_z {
                            map_get_f64(pos_obj, "dPosZ").unwrap_or(0.0)
                        } else {
                            0.0
                        };
                        let pfs_offset = map_get_f64(pos_obj, "dPFSOffset")
                            .and_then(|v| (v >= 0.0).then_some(v));
                        let name = pos_obj
                            .get("dPosName")
                            .or_else(|| pos_obj.get("pPosName"))
                            .or_else(|| pos_obj.get("wszName"))
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

            let count = if points.is_empty() {
                loop_count
            } else {
                points.len() as u32
            };
            Ok(Some(ExpLoop::XYPosLoop(XYPosLoop {
                count,
                nesting_level,
                parameters: XYPosLoopParams {
                    is_setting_z,
                    points,
                },
            })))
        }
        Some(8) => {
            // NETimeLoop
            let mut periods = Vec::new();
            let period_src = params.get("pPeriod");
            let valid: Vec<bool> = params
                .get("pPeriodValid")
                .and_then(|v| match v {
                    ClxValue::Object(m) => {
                        let mut keys: Vec<_> = m.keys().collect();
                        keys.sort();
                        Some(
                            keys.into_iter()
                                .filter_map(|k| m.get(k))
                                .map(|x| value_as_bool(x).unwrap_or(true))
                                .collect(),
                        )
                    }
                    ClxValue::Array(arr) => Some(
                        arr.iter()
                            .map(|x| value_as_bool(x).unwrap_or(true))
                            .collect(),
                    ),
                    _ => None,
                })
                .unwrap_or_default();

            if let Some(period_src) = period_src {
                let period_items: Vec<&ClxValue> = match period_src {
                    ClxValue::Array(arr) => arr.iter().collect(),
                    ClxValue::Object(map) => {
                        let mut keys: Vec<_> = map.keys().collect();
                        keys.sort();
                        keys.into_iter().filter_map(|k| map.get(k)).collect()
                    }
                    _ => Vec::new(),
                };
                for (i, period_item) in period_items.into_iter().enumerate() {
                    if !valid.is_empty() && (i >= valid.len() || !valid[i]) {
                        continue;
                    }
                    if let Some(period_obj) = period_item.as_object() {
                        let count = map_get_u32(period_obj, "uiCount").unwrap_or(0);
                        if count == 0 {
                            continue;
                        }
                        periods.push(Period {
                            count,
                            start_ms: map_get_f64(period_obj, "dStart").unwrap_or(0.0),
                            period_ms: map_get_f64(period_obj, "dPeriod")
                                .or_else(|| map_get_f64(period_obj, "dAvgPeriodDiff"))
                                .unwrap_or(0.0),
                            duration_ms: map_get_f64(period_obj, "dDuration").unwrap_or(0.0),
                            period_diff: None,
                        });
                    }
                }
            }
            let count = periods.iter().map(|p| p.count).sum();
            Ok(Some(ExpLoop::NETimeLoop(NETimeLoop {
                count,
                nesting_level,
                parameters: NETimeLoopParams { periods },
            })))
        }
        Some(7) => Ok(Some(ExpLoop::CustomLoop(CustomLoop {
            count: loop_count,
            nesting_level,
        }))),
        _ => Ok(None),
    }
}
