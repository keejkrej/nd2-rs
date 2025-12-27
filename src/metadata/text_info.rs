use crate::error::Result;
use crate::parse::ClxValue;
use crate::types::TextInfo;

pub fn parse_text_info(clx: ClxValue) -> Result<TextInfo> {
    let obj = match clx.as_object() {
        Some(o) => o,
        None => return Ok(TextInfo::default()),
    };

    let get_str = |key: &str| -> Option<String> {
        obj.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    };

    Ok(TextInfo {
        image_id: get_str("ImageId"),
        info_type: get_str("Type"),
        group: get_str("Group"),
        sample_id: get_str("SampleId"),
        author: get_str("Author"),
        description: get_str("Description"),
        capturing: get_str("Capturing"),
        sampling: get_str("Sampling"),
        location: get_str("Location"),
        date: get_str("Date"),
        conclusion: get_str("Conclusion"),
        info1: get_str("Info1"),
        info2: get_str("Info2"),
        optics: get_str("Optics"),
        app_version: get_str("AppVersion"),
    })
}
