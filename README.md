# nd2-rs

Pure Rust library for reading Nikon ND2 microscopy files (v2.0, v2.1, v3.0).

- Metadata: `attributes()`, `text_info()`, `experiment()`, `sizes()`, `loop_indices()`
- Pixel access: `read_frame(sequence_index)` and `read_frame_2d(p, t, c, z)`
- Encodings: uncompressed and zlib-compressed `ImageDataSeq`

## Installation

```toml
[dependencies]
nd2-rs = "0.1.5"
```

## Usage

```rust
use nd2_rs::{Nd2File, Result};

fn main() -> Result<()> {
    let mut nd2 = Nd2File::open("image.nd2")?;
    let sizes = nd2.sizes()?;
    let pixels = nd2.read_frame_2d(0, 0, 0, 0)?;
    let frame = nd2.read_frame(12)?; // sequence index
    Ok(())
}
```

## CLI

```bash
nd2-rs info image.nd2
nd2-rs frame image.nd2 frame.tif --p 0 --t 0 --c 0 --z 0
```

`info` prints concise dataset shape JSON:

```json
{
  "positions": 132,
  "frames": 181,
  "channels": 3,
  "height": 2044,
  "width": 2048
}
```

`frame` supports writing a single channel to 16-bit TIFF by sequence or by `(p, t, c, z)`.

Recent fixes improved compatibility with ND2 files that:
- store channels in-pixel instead of as separate sequence chunks
- use padded uncompressed row strides via `uiWidthBytes`
- expose `ImageDataSeq` chunk sizes in the file map that do not match the on-disk chunk header
- number sequence chunks with position as the outer loop and time as the fast loop

## Error reporting

`Nd2Error` is now grouped by source:
- `File` for malformed/invalid file contents
- `Input` for user-provided indices and arguments
- `Internal` for internal arithmetic/logic issues
- `Unsupported` for unsupported ND2/CLX variants

## Docs

- [DATASTRUCTURE.md](DATASTRUCTURE.md) – format details and parsing

## References

Inspired by the Python [nd2 library](https://github.com/tlambert03/nd2).

## License

MIT OR Apache-2.0
