use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

use crate::constants::ND2_CHUNK_MAGIC;
use crate::error::{Nd2Error, Result};

/// Chunk header structure (16 bytes)
#[derive(Debug, Clone)]
pub struct ChunkHeader {
    pub magic: u32,
    pub name_length: u32,
    pub data_length: u64,
}

impl ChunkHeader {
    /// Read chunk header from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let magic = reader.read_u32::<LittleEndian>().map_err(|e| {
            Nd2Error::InvalidFormat(format!("Failed to read chunk magic: {}", e))
        })?;
        let name_length = reader.read_u32::<LittleEndian>().map_err(|e| {
            Nd2Error::InvalidFormat(format!("Failed to read chunk name length: {}", e))
        })?;
        let data_length = reader.read_u64::<LittleEndian>().map_err(|e| {
            Nd2Error::InvalidFormat(format!("Failed to read chunk data length: {}", e))
        })?;

        Ok(Self {
            magic,
            name_length,
            data_length,
        })
    }

    /// Validate the magic number
    pub fn validate_magic(&self) -> Result<()> {
        if self.magic != ND2_CHUNK_MAGIC {
            return Err(Nd2Error::InvalidMagic {
                expected: ND2_CHUNK_MAGIC,
                actual: self.magic,
            });
        }
        Ok(())
    }
}
