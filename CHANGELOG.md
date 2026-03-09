# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.1.5] - 2026-03-09

### Fixed

- **Raw `ImageDataSeq` reads**: Use the on-disk chunk header for uncompressed frame payloads instead of trusting the file map size field. This fixes frame extraction for files where the chunk map entry size is not a valid raw byte count.
- **Sequence index mapping**: Reverse experiment loop order for chunk indexing so `(p, t, c, z)` resolves to the correct `ImageDataSeq` on multi-position time series such as `250129_HuH7.nd2`.
- **Integration coverage**: Align tests with `sequence_count` rather than assuming sequence chunks always equal `P*T*C*Z` when channels are stored in-pixel.

## [0.1.4] - 2026-03-07

### Fixed

- **Uncompressed frame extraction**: Read `ImageDataSeq` pixel data from the raw frame payload offset used by Nikon/nd2-py instead of requiring a valid chunk header at the filemap offset. This prevents conversion failures on ND2 files with zeroed image chunk headers.
- **Padded row strides**: Respect `width_bytes` when decoding uncompressed frames so files with padded rows are reshaped correctly.

## [0.1.2] - 2025-02-21

### Fixed

- **ChunkNotFound when channels are in-pixel**: For ND2 files where `sequence_count` equals the product of experiment loops (e.g. P×T only, no C in sequence), do not include C in axis order for chunk lookup. Each ImageDataSeq chunk stores one (P,T) frame with all channels; including C produced seq_index 0..1079 but file only has 0..359 chunks.

## [0.1.1] - 2025-02-21

### Fixed

- **Experiment parsing (nd2-py compatibility)**: Major fixes so multi-position/time files (e.g. P=2, T=180) parse correctly and `read_frame_2d` returns the right frames
  - CLX Lite: ByteArray nested-parsing guard (`name_length > 1`) so fields like `pItemValid` are no longer misdetected as nested CLX
  - Experiment structure: unwrap single-key objects (`i0000000000`, `SLxExperiment`, `""`) and single-element arrays
  - ppNextLevelEx: handle direct loop object vs indexed children
  - XYPosLoop: use `uLoopPars` for params, `uiCount` for count, `Points`/`pPeriod` for positions; filter by `pItemValid`
  - TimeLoop: read `dStart`, `dPeriod`, `dDuration` from `uLoopPars` (fix incorrect multipliers)
  - Sequence index: derive axis order from experiment loops; `read_frame_2d` extracts correct channel from planar frame

### Changed

- CLI: `info <file>` now always outputs JSON (removed `--json` flag)
- Docs: renamed ARCHITECTURE.md to DATASTRUCTURE.md

### Removed

- CLI: `chunks` subcommand (use library `chunk_names()` for debugging)

## [0.1.0] - 2025-02-21

### Added

- Initial release
- Metadata: `attributes()`, `text_info()`, `experiment()`
- `sizes()` – dimension sizes (P, T, C, Z, Y, X) from attributes and experiment
- `loop_indices()` – sequence index → (P, T, C, Z) mapping in row-major order
- `read_frame(index)` – read raw u16 pixels (C×Y×X) by sequence index
- `read_frame_2d(p, t, c, z)` – read 2D Y×X frame at (P, T, C, Z)
- Support for uncompressed and zlib-compressed image data
- CLI: `info <file>`, `info <file> --json`, `chunks <file>`
- GitHub Actions CI: build/test (ubuntu, macos, windows), clippy
