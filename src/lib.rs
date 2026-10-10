#![doc = include_str!("../README.md")]

mod error;
mod io;
mod types;

mod chunk;
mod constants;
#[path = "metadata/mod.rs"]
mod meta_parse;
mod parse;
mod reader;

pub use error::{
    ErrorSource, FileError, InputError, InternalError, Nd2Error, Result, UnsupportedError,
};
pub use io::ReadSeek;
pub use reader::Nd2File;
pub use types::{DatasetSummary, SummaryChannel, SummaryScaling};

/// Type alias for [`Nd2File`] for ergonomic and generic usage across reader libraries.
pub type Reader = Nd2File;
