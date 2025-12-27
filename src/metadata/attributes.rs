use crate::error::{Nd2Error, Result};
use crate::parse::ClxValue;
use crate::types::{Attributes, CompressionType, PixelDataType};

pub fn parse_attributes(clx: ClxValue) -> Result<Attributes> {
    let obj = clx
        .as_object()
        .ok_or_else(|| Nd2Error::MetadataParse("Expected object for attributes".to_string()))?;

    let get_u32 = |key: &str| -> Result<u32> {
        obj.get(key)
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .ok_or_else(|| Nd2Error::MetadataParse(format!("Missing or invalid {}", key)))
    };

    let get_opt_u32 = |key: &str| -> Option<u32> {
        obj.get(key).and_then(|v| v.as_u64()).map(|v| v as u32)
    };

    let get_opt_f64 = |key: &str| -> Option<f64> { obj.get(key).and_then(|v| v.as_f64()) };

    let pixel_data_type = obj
        .get("uiCompBPC")
        .and_then(|v| v.as_u64())
        .map(|v| {
            if v == 3 {
                PixelDataType::Float
            } else {
                PixelDataType::Unsigned
            }
        })
        .unwrap_or(PixelDataType::Unsigned);

    let compression_type = obj.get("eCompression").and_then(|v| v.as_str()).map(|s| {
        match s {
            "lossless" => CompressionType::Lossless,
            "lossy" => CompressionType::Lossy,
            _ => CompressionType::None,
        }
    });

    Ok(Attributes {
        bits_per_component_in_memory: get_u32("uiBpcInMemory")?,
        bits_per_component_significant: get_u32("uiBpcSignificant")?,
        component_count: get_u32("uiComp")?,
        height_px: get_u32("uiHeight")?,
        pixel_data_type,
        sequence_count: get_u32("uiSequenceCount")?,
        width_bytes: get_opt_u32("uiWidthBytes"),
        width_px: get_opt_u32("uiWidth"),
        compression_level: get_opt_f64("dCompressionParam"),
        compression_type,
        tile_height_px: get_opt_u32("uiTileHeight"),
        tile_width_px: get_opt_u32("uiTileWidth"),
        channel_count: get_opt_u32("uiChannelCount"),
    })
}
