use thiserror::Error;

#[derive(Error, Debug)]
pub enum Nd2Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid ND2 file: {0}")]
    InvalidFormat(String),

    #[error("Invalid magic number: expected 0x{expected:08X}, got 0x{actual:08X}")]
    InvalidMagic { expected: u32, actual: u32 },

    #[error("Corrupt chunk header at position {position}")]
    CorruptChunkHeader { position: u64 },

    #[error("Chunk '{name}' not found in chunkmap")]
    ChunkNotFound { name: String },

    #[error("Invalid chunkmap signature")]
    InvalidChunkmapSignature,

    #[error("CLX parsing error: {0}")]
    ClxParse(String),

    #[error("Unsupported CLX data type: {0}")]
    UnsupportedClxType(u8),

    #[error("Decompression error: {0}")]
    Decompression(String),

    #[error("UTF-16 decoding error: {0}")]
    Utf16Decode(String),

    #[error("Unsupported file version: {major}.{minor}")]
    UnsupportedVersion { major: u32, minor: u32 },

    #[error("Metadata parse error: {0}")]
    MetadataParse(String),
}

pub type Result<T> = std::result::Result<T, Nd2Error>;
