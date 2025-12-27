# nd2-rs

Pure Rust library for reading Nikon ND2 microscopy files.

## Overview

`nd2-rs` is a Rust implementation for reading metadata from modern ND2 files (versions 2.0, 2.1, and 3.0) created by Nikon NIS Elements software. This library focuses on metadata extraction and does not yet support image data decoding.

## Features

- Read ND2 file metadata
- Support for modern ND2 formats (v2.0, v2.1, v3.0)
- CLX Lite binary format parser
- Zlib decompression support
- Serde serialization support

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
nd2-rs = "0.1.0"
```

## Usage

```rust
use nd2_rs::{Nd2File, Result};

fn main() -> Result<()> {
    let mut nd2 = Nd2File::open("image.nd2")?;

    // Get file version
    println!("Version: {:?}", nd2.version());

    // Get image attributes
    let attrs = nd2.attributes()?;
    println!("Dimensions: {}x{}", attrs.width_px.unwrap_or(0), attrs.height_px);
    println!("Channels: {}", attrs.component_count);
    println!("Frames: {}", attrs.sequence_count);

    // Get text info
    let text_info = nd2.text_info()?;
    if let Some(description) = &text_info.description {
        println!("Description: {}", description);
    }

    // Get experiment loops (time-lapse, z-stack, etc.)
    let experiment = nd2.experiment()?;
    for exp_loop in experiment {
        println!("{:?}", exp_loop);
    }

    Ok(())
}
```

## CLI Usage

The `nd2-rs` command-line tool provides quick access to ND2 file metadata:

```bash
# Display file information
nd2-rs --input image.nd2 --info

# Output as JSON
nd2-rs --input image.nd2 --info --json

# List all chunks in the file
nd2-rs --input image.nd2 --chunks
```

### Installation

```bash
cargo install --path .
```

Or run directly:

```bash
cargo run -- --input path/to/file.nd2 --info
```

## Library Usage Examples

Run the metadata reader example:

```bash
cargo run --example read_metadata path/to/your/file.nd2
```

## Metadata Types

The library provides strongly-typed access to:

- **Attributes**: Image dimensions, bit depth, pixel type, compression
- **TextInfo**: Author, description, date, and other text metadata
- **Experiment**: Time-lapse, Z-stack, XY position, and custom loop definitions
- **Channels**: Channel names, colors, wavelengths (coming soon)

## File Format

ND2 files use a chunk-based binary format:

- **Chunk Header**: 16 bytes (magic, name_length, data_length)
- **ChunkMap**: Located at end of file, maps chunk names to (offset, size)
- **Metadata Encoding**: CLX Lite binary TLV format or XML (version dependent)
- **Compression**: Zlib for both metadata and image data

For detailed technical documentation about the file format and how the library works, see [ARCHITECTURE.md](ARCHITECTURE.md).

## Limitations

This is an early version focused on metadata readout. Not yet implemented:

- Image data decoding
- Legacy ND2 format (JPEG2000-based, v1.0)
- ROI and binary mask data
- Frame-level metadata
- Channel metadata parsing

## Documentation

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Detailed technical documentation about ND2 file format and parsing
- **[AGENTS.md](AGENTS.md)** - Development notes for Claude Code agents

## License

MIT OR Apache-2.0

## References

This is a Rust reimplementation inspired by the Python [nd2 library](https://github.com/tlambert03/nd2).
