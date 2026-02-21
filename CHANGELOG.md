# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

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
