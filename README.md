# nd2-rs

Pure Rust library for reading Nikon ND2 microscopy files (v2.0, v2.1, v3.0).

- Metadata: `version()` and `summary()`
- Pixel access: `read_frame(sequence_index)` and `read_frame_2d(p, t, c, z)`
- Encodings: uncompressed and zlib-compressed `ImageDataSeq`

## Installation

```toml
[dependencies]
nd2-rs = "0.2.0"
```

## Usage

```rust
use nd2_rs::{Nd2File, Result};

fn main() -> Result<()> {
    let mut nd2 = Nd2File::open("image.nd2")?;
    let summary = nd2.summary()?;
    let pixels = nd2.read_frame_2d(0, 0, 0, 0)?;
    let frame = nd2.read_frame(12)?; // sequence index
    let sizes = &summary.sizes;
    println!("width: {}", sizes["X"]);
    println!("plane pixels: {}", pixels.len());
    println!("frame pixels: {}", frame.len());
    println!("logical frames: {}", summary.logical_frame_count);
    Ok(())
}
```

Recent fixes improved compatibility with ND2 files that:
- store channels in-pixel instead of as separate sequence chunks
- use padded uncompressed row strides via `uiWidthBytes`
- expose `ImageDataSeq` chunk sizes in the file map that do not match the on-disk chunk header
- have missing or zeroed `ImageDataSeq` chunk headers, in which case the reader falls back to Nikon's `4096`-byte image payload offset

## Error reporting

`Nd2Error` is now grouped by source:
- `File` for malformed/invalid file contents
- `Input` for user-provided indices and arguments
- `Internal` for internal arithmetic/logic issues
- `Unsupported` for unsupported ND2/CLX variants

## Docs

- [DATASTRUCTURE.md](DATASTRUCTURE.md) – format details and parsing

## Scope

`nd2-rs` is intentionally library-only. End-user conversion and CLI workflows
belong in companion tooling rather than this crate.

## References

Inspired by the Python [nd2 library](https://github.com/tlambert03/nd2).

## License

MIT OR Apache-2.0
