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
    reader.seek(SeekFrom::End(-40))?;

    let mut signature = [0u8; 32];
    reader.read_exact(&mut signature)?;

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
    reader.read_exact(&mut name)?;

    if &name != ND2_FILEMAP_SIGNATURE {
        return Err(Nd2Error::InvalidFormat(
            "Invalid chunkmap section name".to_string(),
        ));
    }

    // Read chunkmap entries
    let mut chunkmap = HashMap::new();
    let mut bytes_read = 0u64;

    while bytes_read < header.data_length {
        // Read until we hit the terminator signature
        let mut chunk_name = Vec::new();
        loop {
            let byte = reader.read_u8()?;
            bytes_read += 1;
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

        // Read offset and size (2 x u64)
        let offset = reader.read_u64::<LittleEndian>()?;
        let size = reader.read_u64::<LittleEndian>()?;
        bytes_read += 16;

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
    reader.read_exact(&mut data)?;

    Ok(data)
}
