# Claude Code Development Notes

This document contains information for Claude Code agents working on the nd2-rs project.

## Project Context

**Project:** nd2-rs - Rust implementation of ND2 file reader
**Language:** Rust (Edition 2021, MSRV 1.70)
**Purpose:** Parse metadata from Nikon ND2 microscopy files
**Status:** ✅ Core metadata reading implemented, image data decoding not yet implemented

---

## Quick Start for Agents

### Building the Project

```bash
cd nd2-rs
cargo build        # Build library and CLI
cargo test         # Run tests (when added)
cargo build --example read_metadata  # Build example
```

### Running the CLI

```bash
# Run directly with cargo
cargo run -- --input path/to/file.nd2 --info

# Or install and run
cargo install --path .
nd2-rs --input path/to/file.nd2 --info

# JSON output
nd2-rs --input path/to/file.nd2 --info --json

# List chunks
nd2-rs --input path/to/file.nd2 --chunks
```

### Running Examples

```bash
cargo run --example read_metadata path/to/file.nd2
```

### Project Structure Reference

```
nd2-rs/
├── src/
│   ├── lib.rs              # Public API entry point
│   ├── main.rs             # CLI binary entry point
│   ├── reader.rs           # Main Nd2File struct
│   ├── error.rs            # Error types
│   ├── constants.rs        # Magic numbers and signatures
│   ├── chunk/              # Binary chunk parsing
│   │   ├── mod.rs
│   │   ├── header.rs       # 16-byte chunk headers
│   │   └── map.rs          # ChunkMap reading
│   ├── parse/              # Format parsers
│   │   ├── mod.rs
│   │   └── clx_lite.rs     # CLX Lite binary TLV parser
│   ├── types/              # Type definitions
│   │   ├── mod.rs
│   │   ├── attributes.rs   # Image attributes
│   │   ├── experiment.rs   # Experiment loop types
│   │   ├── metadata.rs     # Channel/volume metadata
│   │   └── text_info.rs    # Text metadata
│   └── metadata/           # CLX → Rust struct conversion
│       ├── mod.rs
│       ├── attributes.rs
│       ├── experiment.rs
│       └── text_info.rs
├── examples/
│   └── read_metadata.rs    # Example program
├── Cargo.toml
├── README.md               # User-facing documentation
├── ARCHITECTURE.md         # Technical documentation
└── AGENTS.md              # This file
```

---

## Development Guidelines

### Code Style

- **Idiomatic Rust**: Follow standard Rust conventions
- **Error handling**: Use `Result<T>` with `?` operator, avoid panics
- **Documentation**: Use `///` for public items, `//` for internal
- **Imports**: Group std, external crates, internal modules
- **Naming**: `snake_case` for functions/variables, `PascalCase` for types

**Example:**
```rust
use std::fs::File;
use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::error::{Nd2Error, Result};
use crate::constants::ND2_CHUNK_MAGIC;

/// Read a chunk header from the given reader.
pub fn read_chunk_header<R: Read>(reader: &mut R) -> Result<ChunkHeader> {
    let magic = reader.read_u32::<LittleEndian>()?;
    if magic != ND2_CHUNK_MAGIC {
        return Err(Nd2Error::InvalidMagic {
            expected: ND2_CHUNK_MAGIC,
            actual: magic,
        });
    }
    // ...
}
```

### Dependencies

**Current dependencies** (keep minimal):
- `thiserror` - Error types
- `byteorder` - Binary I/O
- `serde` - Serialization framework
- `flate2` - Zlib decompression

**Dev dependencies:**
- `serde_json` - JSON serialization for examples

**When adding dependencies:**
1. Check if it's truly needed (prefer std when possible)
2. Verify license compatibility (MIT/Apache-2.0)
3. Check maintenance status (last update, open issues)
4. Add to appropriate section in `Cargo.toml`

### Error Handling

**Use `thiserror` for error types:**
```rust
#[derive(Error, Debug)]
pub enum Nd2Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid magic number: expected 0x{expected:08X}, got 0x{actual:08X}")]
    InvalidMagic { expected: u32, actual: u32 },
}
```

**Return `Result<T>` everywhere:**
```rust
pub fn parse_attributes(clx: ClxValue) -> Result<Attributes> {
    let obj = clx.as_object()
        .ok_or_else(|| Nd2Error::MetadataParse("Expected object".to_string()))?;
    // ...
}
```

**Avoid unwrap/expect:**
```rust
// ❌ Bad
let value = map.get("key").unwrap();

// ✅ Good
let value = map.get("key")
    .ok_or_else(|| Nd2Error::MetadataParse("Missing key".to_string()))?;
```

### Testing Strategy

**Unit tests** (when added):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_header_parsing() {
        let data = vec![0xDA, 0xCE, 0xBE, 0x0A, ...];
        let header = ChunkHeader::read(&mut Cursor::new(data)).unwrap();
        assert_eq!(header.magic, ND2_CHUNK_MAGIC);
    }
}
```

**Integration tests** (future):
```rust
// tests/integration.rs
use nd2_rs::Nd2File;

#[test]
fn test_read_attributes() {
    let mut nd2 = Nd2File::open("tests/fixtures/test.nd2").unwrap();
    let attrs = nd2.attributes().unwrap();
    assert_eq!(attrs.width_px, Some(512));
}
```

---

## Common Tasks

### Adding a New Metadata Type

**Example: Adding ROI (Region of Interest) support**

1. **Define the type** in `src/types/roi.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ROI {
    pub id: u32,
    pub name: String,
    pub shape_type: ShapeType,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeType {
    Rectangle,
    Ellipse,
    Polygon,
}
```

2. **Add parser** in `src/metadata/roi.rs`:
```rust
use crate::error::{Nd2Error, Result};
use crate::parse::ClxValue;
use crate::types::ROI;

pub fn parse_rois(clx: ClxValue) -> Result<Vec<ROI>> {
    let obj = clx.as_object()
        .ok_or_else(|| Nd2Error::MetadataParse("Expected object".to_string()))?;

    let mut rois = Vec::new();
    // ... parsing logic
    Ok(rois)
}
```

3. **Export from modules**:
```rust
// src/types/mod.rs
pub mod roi;
pub use roi::*;

// src/metadata/mod.rs
pub mod roi;
pub use roi::*;
```

4. **Add to Nd2File** in `src/reader.rs`:
```rust
pub struct Nd2File {
    // ... existing fields
    rois: Option<Vec<ROI>>,
}

impl Nd2File {
    pub fn rois(&mut self) -> Result<&Vec<ROI>> {
        if self.rois.is_none() {
            let chunk_name: &[u8] = b"CustomData|RoiMetadata_v1!";
            let data = read_chunk(&mut self.reader, &self.chunkmap, chunk_name)?;
            let parser = ClxLiteParser::new(false);
            let clx = parser.parse(&data)?;
            self.rois = Some(parse_rois(clx)?);
        }
        Ok(self.rois.as_ref().unwrap())
    }
}
```

### Adding Binary Image Data Support

**Current status:** Not implemented
**When implementing:**

1. Add `ImageData` type in `src/types/image.rs`
2. Handle compression types:
   - Uncompressed (raw bytes)
   - Lossless (zlib-compressed)
   - Lossy (custom Nikon format)
3. Consider using `ndarray` crate for array handling
4. Add frame reading API:
   ```rust
   pub fn read_frame(&mut self, index: usize) -> Result<ImageData>
   ```

### Performance Optimization

**When performance becomes an issue:**

1. **Memory-mapped I/O:**
   ```rust
   // Add to Cargo.toml: memmap2 = "0.9"
   use memmap2::Mmap;

   let file = File::open(path)?;
   let mmap = unsafe { Mmap::map(&file)? };
   ```

2. **Parallel chunk loading:**
   ```rust
   // Add to Cargo.toml: rayon = "1.10"
   use rayon::prelude::*;
   ```

3. **Reduce allocations:**
   - Reuse buffers
   - Use `&str` instead of `String` where possible
   - Consider `Cow<'a, str>` for mixed owned/borrowed strings

---

## Debugging

### Enable Detailed Error Messages

```rust
// In development, use unwrap() alternatives that show context
let value = map.get("key")
    .expect("Missing key 'key' in attributes");
```

### Inspect Binary Data

```rust
// Dump chunk data to file
let data = nd2.read_raw_chunk(b"ImageAttributesLV!")?;
std::fs::write("chunk_dump.bin", data)?;

// Print hex dump
for chunk in data.chunks(16) {
    println!("{:02X?}", chunk);
}
```

### Debug CLX Parsing

```rust
use nd2_rs::parse::{ClxLiteParser, ClxValue};

let parser = ClxLiteParser::new(false);
let clx = parser.parse(&data)?;

// Pretty-print with Debug
println!("{:#?}", clx);

// Or serialize to JSON
#[cfg(feature = "json")]
{
    println!("{}", serde_json::to_string_pretty(&clx)?);
}
```

---

## Git Workflow

### Commit Messages

Follow conventional commits:
```
feat: add ROI metadata parsing
fix: correct chunk offset calculation
docs: update architecture documentation
test: add CLX parser unit tests
refactor: simplify experiment loop parsing
perf: use memory-mapped I/O for large files
```

### Branch Naming

```
feature/roi-support
fix/chunk-alignment-issue
docs/api-examples
```

---

## API Design Principles

### 1. Lazy Loading with Caching

```rust
// ✅ Good: Load on first access, cache result
pub fn attributes(&mut self) -> Result<&Attributes> {
    if self.attributes.is_none() {
        self.attributes = Some(load_attributes(...)?);
    }
    Ok(self.attributes.as_ref().unwrap())
}

// ❌ Bad: Load every time
pub fn attributes(&mut self) -> Result<Attributes> {
    load_attributes(...)
}
```

### 2. Borrow When Possible

```rust
// ✅ Good: Return reference to cached data
pub fn attributes(&mut self) -> Result<&Attributes>

// ❌ Bad: Clone on every call
pub fn attributes(&mut self) -> Result<Attributes>
```

### 3. Error Context

```rust
// ✅ Good: Specific error with context
.ok_or_else(|| Nd2Error::MetadataParse("Missing uiWidth field".to_string()))?

// ❌ Bad: Generic error
.ok_or_else(|| Nd2Error::MetadataParse("Parse error".to_string()))?
```

### 4. Type Safety

```rust
// ✅ Good: Use enums for fixed values
pub enum CompressionType {
    Lossless,
    Lossy,
    None,
}

// ❌ Bad: Use strings
pub compression_type: String  // Could be "lossless", "Lossless", "LOSSLESS", etc.
```

---

## Known Limitations

### Current Implementation

- ✅ Metadata reading (attributes, text info, experiment)
- ✅ Modern ND2 format (v2.0, v2.1, v3.0)
- ✅ CLX Lite binary parser
- ✅ Zlib decompression
- ❌ Image data decoding
- ❌ Legacy format (v1.0 JPEG2000)
- ❌ ROI metadata
- ❌ Binary masks
- ❌ Frame-level metadata
- ❌ Channel metadata (partially implemented in types but not parsed)

### Platform Support

- **Tested:** Windows (MINGW)
- **Should work:** Linux, macOS (not tested)
- **Endianness:** Little-endian assumed (x86/x64/ARM)

---

## Future Roadmap

### Phase 1: Complete Metadata (Current)
- ✅ Attributes
- ✅ Text info
- ✅ Experiment loops
- 🔲 Channel metadata
- 🔲 ROI data
- 🔲 Binary masks

### Phase 2: Image Data
- 🔲 Uncompressed frames
- 🔲 Zlib-compressed frames
- 🔲 Multi-channel images
- 🔲 Z-stacks and time series

### Phase 3: Advanced Features
- 🔲 Memory-mapped I/O
- 🔲 Parallel chunk loading
- 🔲 Streaming API
- 🔲 Write support (create ND2 files)

### Phase 4: Ecosystem
- 🔲 Python bindings (PyO3)
- 🔲 CLI tool
- 🔲 WebAssembly support
- 🔲 Integration with image processing libraries

---

## Useful Commands

```bash
# Build
cargo build
cargo build --release

# Test
cargo test
cargo test -- --nocapture  # Show println! output

# Check without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy

# Documentation
cargo doc --open

# Benchmarks (when added)
cargo bench

# Tree view of dependencies
cargo tree
```

---

## Reference Files

When implementing new features, refer to these Python source files:

| Component | Python Reference |
|-----------|------------------|
| Chunk parsing | `nd2/src/nd2/_parse/_chunk_decode.py` |
| CLX Lite | `nd2/src/nd2/_parse/_clx_lite.py` |
| CLX XML | `nd2/src/nd2/_parse/_clx_xml.py` |
| Data structures | `nd2/src/nd2/structures.py` |
| SDK types | `nd2/src/nd2/_sdk_types.py` |
| Modern reader | `nd2/src/nd2/_readers/_modern/modern_reader.py` |
| Main API | `nd2/src/nd2/_nd2file.py` |

---

## Questions & Clarifications

### Why separate `types/` and `metadata/` modules?

- `types/`: Pure data structures (Rust types)
- `metadata/`: Conversion logic (ClxValue → Rust types)

This separation allows:
- Types to be used independently of parsing
- Multiple parsers for the same type (XML, binary)
- Clear boundary between data and logic

### Why `&mut self` for getters?

Metadata is loaded lazily and cached. The first call loads data (mutation), subsequent calls return cached reference.

### Why not use `once_cell` or `lazy_static`?

Could be added later for true lazy initialization, but current caching approach is simpler and sufficient.

### Why `BufReader` instead of `mmap`?

- Simpler implementation
- Works on all platforms
- Good enough performance for metadata
- Can add `mmap` later for image data

---

## Contact & Resources

- **Repository:** (Add GitHub URL when available)
- **Python nd2:** https://github.com/tlambert03/nd2
- **Rust docs:** https://doc.rust-lang.org/
- **Serde:** https://serde.rs/
