# nd2-rs

Pure Rust library for reading Nikon ND2 microscopy files (v2.0, v2.1, v3.0).

## Installation

```toml
[dependencies]
nd2-rs = "0.1"
```

## Usage

```rust
use nd2_rs::{Nd2File, Result};

fn main() -> Result<()> {
    let mut nd2 = Nd2File::open("image.nd2")?;
    let sizes = nd2.sizes()?;
    let pixels = nd2.read_frame_2d(0, 0, 0, 0)?;
    Ok(())
}
```

## CLI

```bash
nd2-rs info image.nd2
```

Prints version, attributes, text_info, and experiment as JSON.

## Docs

- [DATASTRUCTURE.md](DATASTRUCTURE.md) – format details and parsing

## References

Inspired by the Python [nd2 library](https://github.com/tlambert03/nd2).

## License

MIT OR Apache-2.0
