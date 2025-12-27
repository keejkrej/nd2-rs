//! nd2-rs: Pure Rust library for reading Nikon ND2 microscopy files
//!
//! This library provides functionality to read metadata from modern ND2 files
//! (versions 2.0, 2.1, and 3.0) created by Nikon NIS Elements software.
//!
//! # Example
//! ```no_run
//! use nd2_rs::Nd2File;
//!
//! fn main() -> nd2_rs::Result<()> {
//!     let mut nd2 = Nd2File::open("image.nd2")?;
//!
//!     println!("Version: {:?}", nd2.version());
//!     println!("Attributes: {:#?}", nd2.attributes()?);
//!
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod types;

mod constants;
mod chunk;
mod parse;
mod metadata;
mod reader;

pub use error::{Nd2Error, Result};
pub use reader::Nd2File;
pub use types::*;
