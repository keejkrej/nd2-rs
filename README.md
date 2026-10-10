# nd2

[![Crates.io](https://img.shields.io/crates/v/nd2.svg)](https://crates.io/crates/nd2)
[![Documentation](https://docs.rs/nd2/badge.svg)](https://docs.rs/nd2)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

A fast, memory-efficient, pure Rust library for reading Nikon ND2 (`.nd2`) microscopy files.

`nd2` provides a clean, ergonomic, high-performance API designed for bioimaging pipelines, image analysis, and scientific computing. It handles modern ND2 formats (versions 2.0, 2.1, and 3.0 created by Nikon NIS Elements), chunk maps, CLX metadata parsing, and decompression without any C/C++ dependencies.

## Features

- **Pure Rust**: 100% safe, portable Rust with zero external C/C++ libraries.
- **Consistent Reader API**: Mirrors the bioimaging API convention established across sister formats (like [`czi`](https://crates.io/crates/czi)).
- **Multi-Dimensional Navigation**: Read frames by linear sequence index or by multidimensional coordinates (Position, Time, Channel, Z-stack).
- **Compression Support**: Handles uncompressed chunks and zlib-compressed `ImageDataSeq` payloads.
- **Broad Format Compatibility**: Robust against in-pixel channel multiplexing, padded row strides (`uiWidthBytes`), unaligned chunk sizes, and headerless sequence blocks.
- **Streaming & In-Memory**: Read directly from the filesystem with `Nd2File::open` or from any in-memory/stream buffer implementing `Read + Seek` with `Nd2File::open_reader`.
- **Categorized Error Handling**: Structured errors classified into `File`, `Input`, `Internal`, and `Unsupported` sources.

## Installation

Add `nd2` to your `Cargo.toml`:

```toml
[dependencies]
nd2 = "1.0.0"
```

## Quick Start

```rust,no_run
use nd2::{Nd2File, Result};

fn main() -> Result<()> {
    // Open an ND2 file from disk
    let mut nd2 = Nd2File::open("sample.nd2")?;

    // Format version
    let (major, minor) = nd2.version();
    println!("ND2 format version: {major}.{minor}");

    // Summary metadata overview
    let summary = nd2.summary()?;
    println!("Dimensions: {:?}", summary.sizes);
    println!("Logical frame count: {}", summary.logical_frame_count);
    println!("Channels: {}", summary.channels.len());

    // Read a 2D plane at (Position=0, Time=0, Channel=0, Z=0)
    let plane_pixels = nd2.read_frame_2d(0, 0, 0, 0)?;
    let width = summary.sizes["X"];
    let height = summary.sizes["Y"];
    assert_eq!(plane_pixels.len(), width * height);

    // Or read by flat sequence index (0..summary.logical_frame_count)
    let first_frame = nd2.read_frame(0)?;
    println!("Read first sequence frame with {} pixels", first_frame.len());

    Ok(())
}
```

## Dimensional Coordinates

Microscopy images commonly record data across multiple experimental dimensions:

| Axis | Meaning |
|:---:|---|
| **X** | Width (columns) |
| **Y** | Height (rows) |
| **C** | Channel (fluorescence wavelength / detector) |
| **Z** | Focal plane (depth) |
| **T** | Time point |
| **P** | Position / Stage point |

`nd2` supports coordinate-based plane extraction via `read_frame_2d(p, t, c, z)`:

```rust,no_run
use nd2::Nd2File;

# fn run() -> nd2::Result<()> {
let mut nd2 = Nd2File::open("timelapse.nd2")?;

// Read position 0, timepoint 3, channel 1, z-slice 5
let plane = nd2.read_frame_2d(0, 3, 1, 5)?;
# Ok(())
# }
```

Decoded 2D planes are returned as `Vec<u16>` in row-major order (`height * width`). Flat frames accessed via `read_frame(index)` return planar `(C, Y, X)` pixel buffers.

## Reading from Memory / Custom Streams

For network streams, archives, or test fixtures, `open_reader` accepts any source implementing `Read + Seek`:

```rust,no_run
use std::io::Cursor;
use nd2::Nd2File;

# fn run(data: Vec<u8>) -> nd2::Result<()> {
let cursor = Cursor::new(data);
let mut nd2 = Nd2File::open_reader(cursor)?;
let summary = nd2.summary()?;
# Ok(())
# }
```

## Error Handling

Errors are returned as [`Nd2Error`](https://docs.rs/nd2/latest/nd2/enum.Nd2Error.html) with source categories:

```rust,no_run
use nd2::{ErrorSource, Nd2Error, Nd2File};

let res = Nd2File::open("nonexistent.nd2");
if let Err(err) = res {
    match err.source() {
        ErrorSource::File => eprintln!("File or corruption error: {err}"),
        ErrorSource::Input => eprintln!("Invalid index or parameter: {err}"),
        ErrorSource::Unsupported => eprintln!("Unsupported feature/version: {err}"),
        ErrorSource::Internal => eprintln!("Internal overflow: {err}"),
    }

    // Or use the ergonomic predicate methods:
    if err.is_file() {
        // Handle filesystem / format errors
    }
}
```

## Documentation

- [DATASTRUCTURE.md](DATASTRUCTURE.md) – Technical details on chunk maps and CLX structures.

## References

Inspired by the Python [nd2 library](https://github.com/tlambert03/nd2).

## License

Dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
