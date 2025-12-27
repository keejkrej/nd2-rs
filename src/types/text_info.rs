use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TextInfo {
    pub image_id: Option<String>,
    #[serde(rename = "type")]
    pub info_type: Option<String>,
    pub group: Option<String>,
    pub sample_id: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub capturing: Option<String>,
    pub sampling: Option<String>,
    pub location: Option<String>,
    pub date: Option<String>,
    pub conclusion: Option<String>,
    pub info1: Option<String>,
    pub info2: Option<String>,
    pub optics: Option<String>,
    pub app_version: Option<String>,
}
