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
    // Read last 40 bytes: 32-byte signature + 8-byte offset
    reader.seek(SeekFrom::End(-40)).map_err(|e| {
        Nd2Error::InvalidFormat(format!(
            "Failed to seek to chunkmap signature (file may be too small): {}",
            e
        ))
    })?;

    let mut signature = [0u8; 32];
    reader.read_exact(&mut signature).map_err(|e| {
        Nd2Error::InvalidFormat(format!(
            "Failed to read chunkmap signature: {}",
            e
        ))
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

    // Read and validate chunkmap name
    let mut name = vec![0u8; header.name_length as usize];
    reader.read_exact(&mut name).map_err(|e| {
        Nd2Error::InvalidFormat(format!("Failed to read chunkmap name: {}", e))
    })?;

    if name != ND2_FILEMAP_SIGNATURE {
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

    // Parse entries from the buffer
    let mut chunkmap = HashMap::new();
    let mut pos = 0usize;

    while pos < chunkmap_data.len() {
        // Check if we have enough bytes left for at least a minimal entry
        if pos + 18 > chunkmap_data.len() {
            break;
        }

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
            if chunk_name.len() >= 32 && &chunk_name[chunk_name.len() - 32..] == ND2_CHUNKMAP_SIGNATURE {
                // We've hit the terminator, done reading
                return Ok(chunkmap);
            }
        }

        // If we didn't get a complete chunk name, break out
        if !chunk_name.ends_with(b"!") {
            break;
        }

        // Read offset and size (2 x u64)
        if pos + 16 > chunkmap_data.len() {
            break;
        }

        let offset = u64::from_le_bytes([
            chunkmap_data[pos], chunkmap_data[pos+1], chunkmap_data[pos+2], chunkmap_data[pos+3],
            chunkmap_data[pos+4], chunkmap_data[pos+5], chunkmap_data[pos+6], chunkmap_data[pos+7],
        ]);
        let size = u64::from_le_bytes([
            chunkmap_data[pos+8], chunkmap_data[pos+9], chunkmap_data[pos+10], chunkmap_data[pos+11],
            chunkmap_data[pos+12], chunkmap_data[pos+13], chunkmap_data[pos+14], chunkmap_data[pos+15],
        ]);
        pos += 16;

        chunkmap.insert(chunk_name, (offset, size));
    }

    Ok(chunkmap)
}

/// Read a chunk's data given the chunkmap
pub fn read_chunk<R: Read + Seek>(
    reader: &mut R,
    chunkmap: &ChunkMap,
    name: &[u8],
) -> Result<Vec<u8>> {
    let (offset, size) = chunkmap
        .get(name)
        .ok_or_else(|| Nd2Error::ChunkNotFound {
            name: String::from_utf8_lossy(name).to_string(),
        })?;

    // Seek to chunk data (skip header + name)
    reader.seek(SeekFrom::Start(*offset))?;

    let header = ChunkHeader::read(reader)?;
    header.validate_magic()?;

    // Skip chunk name
    reader.seek(SeekFrom::Current(header.name_length as i64))?;

    // Read chunk data
    let mut data = vec![0u8; *size as usize];
    reader.read_exact(&mut data).map_err(|e| {
        Nd2Error::InvalidFormat(format!(
            "Failed to read chunk data for '{}': {}",
            String::from_utf8_lossy(name),
            e
        ))
    })?;

    Ok(data)
}
