use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

use crate::chunk::ChunkHeader;
use crate::constants::{ND2_CHUNKMAP_SIGNATURE, ND2_FILEMAP_SIGNATURE};
use crate::error::{Nd2Error, Result};

/// ChunkMap: mapping of chunk names to (offset, size) pairs
pub type ChunkMap = HashMap<Vec<u8>, (u64, u64)>;

/// Read the chunkmap from the end of the file
pub fn read_chunkmap<R: Read + Seek>(reader: &mut R) -> Result<ChunkMap> {
    let file_size = reader.seek(SeekFrom::End(0))?;

    // Read last 40 bytes: 32-byte signature + 8-byte offset
    reader.seek(SeekFrom::End(-40)).map_err(|e| {
        Nd2Error::InvalidFormat(format!(
            "Failed to seek to chunkmap signature (file may be too small): {}",
            e
        ))
    })?;

    let mut signature = [0u8; 32];
    reader.read_exact(&mut signature).map_err(|e| {
        Nd2Error::InvalidFormat(format!("Failed to read chunkmap signature: {}", e))
    })?;

    if &signature != ND2_CHUNKMAP_SIGNATURE {
        return Err(Nd2Error::InvalidChunkmapSignature);
    }

    let chunkmap_offset = reader.read_u64::<LittleEndian>()?;

    // Seek to chunkmap section
    reader.seek(SeekFrom::Start(chunkmap_offset))?;

    // Read chunkmap header
    let header = ChunkHeader::read(reader)?;
    header.validate_magic()?;

    // Read and validate chunkmap name (supports optional zero padding)
    let mut name = vec![0u8; header.name_length as usize];
    reader
        .read_exact(&mut name)
        .map_err(|e| Nd2Error::InvalidFormat(format!("Failed to read chunkmap name: {}", e)))?;

    let is_expected_name = name.starts_with(ND2_FILEMAP_SIGNATURE)
        && name[ND2_FILEMAP_SIGNATURE.len()..]
            .iter()
            .all(|byte| *byte == 0);

    if !is_expected_name {
        return Err(Nd2Error::InvalidFormat(
            "Invalid chunkmap section name".to_string(),
        ));
    }

    // Read chunkmap entries
    // Read all chunkmap data into a buffer to avoid EOF issues with BufReader
    let mut chunkmap_data = vec![0u8; header.data_length as usize];
    reader.read_exact(&mut chunkmap_data).map_err(|e| {
        Nd2Error::InvalidFormat(format!(
            "Failed to read {} bytes of chunkmap data: {}",
            header.data_length, e
        ))
    })?;

    // Parse entries from the buffer.
    let mut chunkmap = HashMap::new();
    let mut pos = 0usize;

    let read_offset_size = |chunkmap_data: &[u8], value_pos: usize| -> Option<(u64, u64)> {
        if value_pos + 16 > chunkmap_data.len() {
            return None;
        }

        let offset = u64::from_le_bytes([
            chunkmap_data[value_pos],
            chunkmap_data[value_pos + 1],
            chunkmap_data[value_pos + 2],
            chunkmap_data[value_pos + 3],
            chunkmap_data[value_pos + 4],
            chunkmap_data[value_pos + 5],
            chunkmap_data[value_pos + 6],
            chunkmap_data[value_pos + 7],
        ]);
        let size = u64::from_le_bytes([
            chunkmap_data[value_pos + 8],
            chunkmap_data[value_pos + 9],
            chunkmap_data[value_pos + 10],
            chunkmap_data[value_pos + 11],
            chunkmap_data[value_pos + 12],
            chunkmap_data[value_pos + 13],
            chunkmap_data[value_pos + 14],
            chunkmap_data[value_pos + 15],
        ]);

        Some((offset, size))
    };

    while pos < chunkmap_data.len() {
        // Read until we hit '!' for chunk name
        let mut chunk_name = Vec::new();
        while pos < chunkmap_data.len() {
            let byte = chunkmap_data[pos];
            pos += 1;
            chunk_name.push(byte);

            if chunk_name.ends_with(b"!") {
                break;
            }

            // Check for terminator signature
            if chunk_name.len() >= 32
                && &chunk_name[chunk_name.len() - 32..] == ND2_CHUNKMAP_SIGNATURE
            {
                // We've hit the terminator, done reading
                return Ok(chunkmap);
            }
        }

        // If we didn't get a complete chunk name, break out
        if !chunk_name.ends_with(b"!") {
            break;
        }

        // End marker is a special terminator chunk entry.
        if chunk_name == ND2_CHUNKMAP_SIGNATURE {
            break;
        }

        let mut value_pos = pos;
        let mut found_entry = false;
        let mut best_score = -1i32;

        // Prefer the candidate with a valid file-bound check when available.
        // Older ND2 files (and some edge cases) may encode this field using offset+1 alignment.
        for candidate in 0..=1 {
            let value = match read_offset_size(&chunkmap_data, pos + candidate) {
                Some(v) => v,
                None => continue,
            };

            let (offset, size) = value;
            let mut score = 0;
            if offset <= file_size {
                score = 1;
                if let Some(end) = offset.checked_add(size) {
                    if end <= file_size {
                        score = 2;
                    }
                }
            }

            if score > best_score {
                best_score = score;
                chunkmap.insert(chunk_name.clone(), value);
                value_pos = pos + candidate;
                found_entry = true;
                if score == 2 {
                    // Best possible score, keep scanning for a better candidate is unnecessary.
                    break;
                }
            }
        }

        if !found_entry {
            return Err(Nd2Error::InvalidFormat(
                "Invalid chunkmap entry offset/size values".to_string(),
            ));
        }

        pos = value_pos + 16;
    }

    Ok(chunkmap)
}

/// Read a chunk's data given the chunkmap
pub fn read_chunk<R: Read + Seek>(
    reader: &mut R,
    chunkmap: &ChunkMap,
    name: &[u8],
) -> Result<Vec<u8>> {
    let file_size = reader.seek(SeekFrom::End(0))?;

    let (offset, map_size) = chunkmap.get(name).ok_or_else(|| Nd2Error::ChunkNotFound {
        name: String::from_utf8_lossy(name).to_string(),
    })?;

    // Seek to chunk data (skip header + name)
    reader.seek(SeekFrom::Start(*offset))?;

    let header = ChunkHeader::read(reader)?;
    header.validate_magic()?;

    // Skip chunk name
    reader.seek(SeekFrom::Current(header.name_length as i64))?;

    let size = header.data_length;
    let chunk_end = (*offset)
        .checked_add(16)
        .and_then(|v| v.checked_add(header.name_length as u64))
        .and_then(|v| v.checked_add(size))
        .ok_or_else(|| {
            Nd2Error::InvalidFormat(format!(
                "Invalid chunk bounds for '{}': offset {} size {}",
                String::from_utf8_lossy(name),
                offset,
                map_size
            ))
        })?;

    if chunk_end > file_size {
        return Err(Nd2Error::InvalidFormat(format!(
            "Invalid chunk bounds for '{}': offset {} size {}",
            String::from_utf8_lossy(name),
            offset,
            map_size
        )));
    }

    let size: usize = size.try_into().map_err(|_| {
        Nd2Error::InvalidFormat(format!(
            "Chunk size {} for '{}' too large for this platform",
            size,
            String::from_utf8_lossy(name)
        ))
    })?;

    // Read chunk data
    let mut data = vec![0u8; size];
    reader.read_exact(&mut data).map_err(|e| {
        Nd2Error::InvalidFormat(format!(
            "Failed to read chunk data for '{}': {}",
            String::from_utf8_lossy(name),
            e
        ))
    })?;

    Ok(data)
}
