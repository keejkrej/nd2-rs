use std::io::{Read, Seek};

/// Type-erased readable/seekable source for ND2 parsing.
pub trait ReadSeek: Read + Seek {}

impl<T: Read + Seek> ReadSeek for T {}
