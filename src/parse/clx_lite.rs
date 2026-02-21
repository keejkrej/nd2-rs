use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use std::collections::HashMap;
use std::io::{Cursor, Read};

use crate::constants::clx_types;
use crate::error::{Nd2Error, Result};

/// Parsed JSON-like value from CLX Lite format
#[derive(Debug, Clone, PartialEq)]
pub enum ClxValue {
    Bool(bool),
    Int(i64),
    UInt(u64),
    Float(f64),
    String(String),
    ByteArray(Vec<u8>),
    Object(HashMap<String, ClxValue>),
    Array(Vec<ClxValue>),
}

impl ClxValue {
    pub fn as_object(&self) -> Option<&HashMap<String, ClxValue>> {
        if let ClxValue::Object(map) = self {
            Some(map)
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let ClxValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        if let ClxValue::Int(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        if let ClxValue::UInt(u) = self {
            Some(*u)
        } else {
            None
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        if let ClxValue::Float(f) = self {
            Some(*f)
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let ClxValue::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }
}

/// Parser for CLX Lite binary TLV format
pub struct ClxLiteParser {
    strip_prefix: bool,
}

impl ClxLiteParser {
    pub fn new(strip_prefix: bool) -> Self {
        Self { strip_prefix }
    }

    /// Parse the entire buffer into a ClxValue
    pub fn parse(&self, data: &[u8]) -> Result<ClxValue> {
        let mut cursor = Cursor::new(data);
        self.parse_with_count(&mut cursor, 1)
    }

    fn parse_with_count(&self, cursor: &mut Cursor<&[u8]>, count: usize) -> Result<ClxValue> {
        let mut output = HashMap::new();

        for _ in 0..count {
            let (name, data_type) = self.read_chunk_header(cursor)?;

            if data_type == -1 {
                break;
            }

            let value = match data_type as u8 {
                clx_types::COMPRESS => {
                    // Skip 10 bytes, decompress rest, parse recursively
                    cursor.set_position(cursor.position() + 10);
                    let mut compressed = Vec::new();
                    cursor.read_to_end(&mut compressed)?;
                    let decompressed = decompress_zlib(&compressed)?;
                    return self.parse(&decompressed);
                }
                clx_types::BOOL => ClxValue::Bool(cursor.read_u8()? != 0),
                clx_types::INT32 => ClxValue::Int(cursor.read_i32::<LittleEndian>()? as i64),
                clx_types::UINT32 => ClxValue::UInt(cursor.read_u32::<LittleEndian>()? as u64),
                clx_types::INT64 => ClxValue::Int(cursor.read_i64::<LittleEndian>()?),
                clx_types::UINT64 => ClxValue::UInt(cursor.read_u64::<LittleEndian>()?),
                clx_types::DOUBLE => ClxValue::Float(cursor.read_f64::<LittleEndian>()?),
                clx_types::VOID_POINTER => ClxValue::UInt(cursor.read_u64::<LittleEndian>()?),
                clx_types::STRING => self.read_utf16_string(cursor)?,
                clx_types::BYTE_ARRAY => self.read_byte_array(cursor)?,
                clx_types::LEVEL => self.read_level(cursor)?,
                other => return Err(Nd2Error::UnsupportedClxType(other)),
            };

            // Handle empty names (list elements in nd2)
            if name.is_empty() {
                if let Some(ClxValue::Array(arr)) = output.get_mut("") {
                    arr.push(value);
                } else if output.contains_key("") {
                    let existing = output.remove("").unwrap();
                    output.insert(String::new(), ClxValue::Array(vec![existing, value]));
                } else {
                    output.insert(String::new(), value);
                }
            } else {
                output.insert(name, value);
            }
        }

        Ok(ClxValue::Object(output))
    }

    fn read_chunk_header(&self, cursor: &mut Cursor<&[u8]>) -> Result<(String, i8)> {
        let data_type = cursor.read_u8()? as i8;
        let name_length = cursor.read_u8()? as usize;

        if data_type == clx_types::DEPRECATED as i8 || data_type == clx_types::UNKNOWN as i8 {
            return Err(Nd2Error::ClxParse(format!(
                "Unknown data type in metadata header: {}",
                data_type
            )));
        }

        let name = if data_type == clx_types::COMPRESS as i8 {
            String::new()
        } else {
            let mut name_bytes = vec![0u8; name_length * 2];
            cursor.read_exact(&mut name_bytes)?;
            let name = decode_utf16_le(&name_bytes)?;
            // Strip null terminator
            let name = name.trim_end_matches('\0');
            if self.strip_prefix {
                strip_lowercase_prefix(name)
            } else {
                name.to_string()
            }
        };

        Ok((name, data_type))
    }

    fn read_utf16_string(&self, cursor: &mut Cursor<&[u8]>) -> Result<ClxValue> {
        // Read 2 bytes at a time until we hit \x00\x00
        let mut bytes = Vec::new();
        loop {
            let b1 = cursor.read_u8()?;
            let b2 = cursor.read_u8()?;
            bytes.push(b1);
            bytes.push(b2);
            if b1 == 0 && b2 == 0 {
                break;
            }
        }
        let s = decode_utf16_le(&bytes)?;
        Ok(ClxValue::String(s.trim_end_matches('\0').to_string()))
    }

    fn read_byte_array(&self, cursor: &mut Cursor<&[u8]>) -> Result<ClxValue> {
        let size = cursor.read_u64::<LittleEndian>()? as usize;
        let mut bytes = vec![0u8; size];
        cursor.read_exact(&mut bytes)?;

        // Try to parse as nested CLX Lite if it looks valid
        if looks_like_clx_lite(&bytes) {
            if let Ok(nested) = self.parse(&bytes) {
                return Ok(nested);
            }
        }

        Ok(ClxValue::ByteArray(bytes))
    }

    fn read_level(&self, cursor: &mut Cursor<&[u8]>) -> Result<ClxValue> {
        let item_count = cursor.read_u32::<LittleEndian>()? as usize;
        let _length = cursor.read_u64::<LittleEndian>()? as usize;

        // Parse the nested data
        let value = self.parse_with_count(cursor, item_count)?;

        // Skip the item_count * 8 bytes of offset data
        cursor.set_position(cursor.position() + (item_count as u64 * 8));

        // Handle the case where all items have empty names (array-like)
        if let ClxValue::Object(ref map) = value {
            if map.len() == 1 && map.contains_key("") {
                if let Some(arr) = map.get("") {
                    return Ok(arr.clone());
                }
            }
        }

        Ok(value)
    }
}

/// Check if data looks like valid CLX Lite
fn looks_like_clx_lite(data: &[u8]) -> bool {
    if data.len() < 2 {
        return false;
    }
    let data_type = data[0];
    let name_length = data[1];

    // Compressed data
    if data_type == clx_types::COMPRESS {
        return true;
    }

    // Valid data types: 1-11
    if !(1..=11).contains(&data_type) {
        return false;
    }

    // Require at least 2 UTF-16 chars for standalone detection.
    // name_length <= 1 is often just empty/null and can falsely match (e.g. pItemValid).
    if name_length <= 1 {
        return false;
    }

    // Check minimum size requirements
    let name_bytes = name_length as usize * 2;
    let header_and_name = 2 + name_bytes;
    if data.len() < header_and_name {
        return false;
    }

    // Verify UTF-16 null terminator
    let name_end = 2 + name_bytes;
    if data[name_end - 2..name_end] != [0, 0] {
        return false;
    }

    true
}

/// Decode UTF-16 LE bytes to String
fn decode_utf16_le(bytes: &[u8]) -> Result<String> {
    let u16s: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16(&u16s).map_err(|e| Nd2Error::Utf16Decode(e.to_string()))
}

/// Strip lowercase prefix from identifier (e.g., "uiWidth" -> "Width")
fn strip_lowercase_prefix(s: &str) -> String {
    s.chars()
        .skip_while(|c| c.is_lowercase() || *c == '_')
        .collect()
}

/// Decompress zlib data
fn decompress_zlib(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| Nd2Error::Decompression(e.to_string()))?;
    Ok(decompressed)
}
