use thiserror::Error;

#[derive(Error, Debug)]
pub enum Nd2Error {
    #[error("file error: {source}")]
    File { source: FileError },

    #[error("input error: {source}")]
    Input { source: InputError },

    #[error("internal error: {source}")]
    Internal { source: InternalError },

    #[error("unsupported: {source}")]
    Unsupported { source: UnsupportedError },
}

#[derive(Error, Debug)]
pub enum FileError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid ND2 file: {context}")]
    InvalidFormat { context: String },

    #[error("Invalid magic number: expected 0x{expected:08X}, got 0x{actual:08X}")]
    InvalidMagic { expected: u32, actual: u32 },

    #[error("Corrupt chunk header at position {position}")]
    CorruptChunkHeader { position: u64 },

    #[error("Chunk '{name}' not found in chunkmap")]
    ChunkNotFound { name: String },

    #[error("Invalid chunkmap signature")]
    InvalidChunkmapSignature,

    #[error("Chunkmap error: {context}")]
    ChunkmapParse { context: String },

    #[error("CLX parsing error: {context}")]
    ClxParse { context: String },

    #[error("Decompression error: {context}")]
    Decompression { context: String },

    #[error("UTF-16 decoding error: {context}")]
    Utf16Decode { context: String },

    #[error("Metadata parse error: {context}")]
    MetadataParse { context: String },
}

#[derive(Error, Debug)]
pub enum InputError {
    #[error("Missing required dimension '{dimension}'")]
    MissingDimension { dimension: String },

    #[error("{field} index out of range: got {index}, max {max}")]
    OutOfRange {
        field: String,
        index: usize,
        max: usize,
    },

    #[error("Invalid input for {field}: {detail}")]
    InvalidArgument { field: String, detail: String },

    #[error("Incompatible parameters: expected {expected}, provided {provided}")]
    IncompatibleParams { expected: String, provided: String },
}

#[derive(Error, Debug)]
pub enum InternalError {
    #[error("Arithmetic overflow during {operation}")]
    Overflow { operation: String },

    #[error("Internal invariant violation: {detail}")]
    InvariantViolation { detail: String },
}

#[derive(Error, Debug)]
pub enum UnsupportedError {
    #[error("Unsupported ND2 file version: {major}.{minor}")]
    Version { major: u32, minor: u32 },

    #[error("Unsupported CLX data type: {type_code}")]
    ClxType { type_code: u8 },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ErrorSource {
    File,
    Input,
    Internal,
    Unsupported,
}

impl Nd2Error {
    pub fn source(&self) -> ErrorSource {
        match self {
            Self::File { .. } => ErrorSource::File,
            Self::Input { .. } => ErrorSource::Input,
            Self::Internal { .. } => ErrorSource::Internal,
            Self::Unsupported { .. } => ErrorSource::Unsupported,
        }
    }

    pub fn is_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }

    pub fn is_input(&self) -> bool {
        matches!(self, Self::Input { .. })
    }

    pub fn is_internal(&self) -> bool {
        matches!(self, Self::Internal { .. })
    }

    pub fn is_unsupported(&self) -> bool {
        matches!(self, Self::Unsupported { .. })
    }

    pub fn file_invalid_format(context: impl Into<String>) -> Self {
        Self::File {
            source: FileError::InvalidFormat {
                context: context.into(),
            },
        }
    }

    pub fn file_chunkmap(context: impl Into<String>) -> Self {
        Self::File {
            source: FileError::ChunkmapParse {
                context: context.into(),
            },
        }
    }

    pub fn file_metadata(context: impl Into<String>) -> Self {
        Self::File {
            source: FileError::MetadataParse {
                context: context.into(),
            },
        }
    }

    pub fn file_invalid_magic(expected: u32, actual: u32) -> Self {
        Self::File {
            source: FileError::InvalidMagic { expected, actual },
        }
    }

    pub fn file_chunk_not_found(name: impl Into<String>) -> Self {
        Self::File {
            source: FileError::ChunkNotFound { name: name.into() },
        }
    }

    pub fn input_out_of_range(field: impl Into<String>, index: usize, max: usize) -> Self {
        Self::Input {
            source: InputError::OutOfRange {
                field: field.into(),
                index,
                max,
            },
        }
    }

    pub fn input_missing_dim(dimension: impl Into<String>) -> Self {
        Self::Input {
            source: InputError::MissingDimension {
                dimension: dimension.into(),
            },
        }
    }

    pub fn input_argument(field: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::Input {
            source: InputError::InvalidArgument {
                field: field.into(),
                detail: detail.into(),
            },
        }
    }

    pub fn internal_overflow(operation: impl Into<String>) -> Self {
        Self::Internal {
            source: InternalError::Overflow {
                operation: operation.into(),
            },
        }
    }

    pub fn unsupported_version(major: u32, minor: u32) -> Self {
        Self::Unsupported {
            source: UnsupportedError::Version { major, minor },
        }
    }

    pub fn unsupported_clx_type(type_code: u8) -> Self {
        Self::Unsupported {
            source: UnsupportedError::ClxType { type_code },
        }
    }
}

impl From<std::io::Error> for Nd2Error {
    fn from(value: std::io::Error) -> Self {
        Self::File {
            source: FileError::Io(value),
        }
    }
}

pub type Result<T> = std::result::Result<T, Nd2Error>;
