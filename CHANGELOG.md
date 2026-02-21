# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

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
