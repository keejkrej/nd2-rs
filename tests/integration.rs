//! Public API integration tests for nd2-rs.
//!
//! Without ND2_TEST_FILE: tests skip (pass).

use nd2_rs::{Nd2File, Result};
use std::path::PathBuf;

fn test_path() -> Option<PathBuf> {
    std::env::var("ND2_TEST_FILE").ok().map(|s| s.into())
}

fn require_fixture() -> Option<Nd2File> {
    let path = test_path()?;
    if !path.exists() {
        return None;
    }
    Nd2File::open(path).ok()
}

#[test]
fn test_version() -> Result<()> {
    let nd2 = match require_fixture() {
        Some(x) => x,
        None => return Ok(()),
    };

    let version = nd2.version();
    assert!(version.0 >= 2 && version.0 <= 3);
    Ok(())
}

#[test]
fn test_summary() -> Result<()> {
    let mut nd2 = match require_fixture() {
        Some(x) => x,
        None => return Ok(()),
    };

    let summary = nd2.summary()?;
    assert!(summary.version_major >= 2);
    assert!(summary.sizes["X"] > 0);
    assert!(summary.sizes["Y"] > 0);
    assert!(summary.logical_frame_count > 0);
    assert_eq!(
        summary.channels.len(),
        *summary.sizes.get("C").unwrap_or(&1)
    );
    Ok(())
}

#[test]
fn test_read_frame() -> Result<()> {
    let mut nd2 = match require_fixture() {
        Some(x) => x,
        None => return Ok(()),
    };

    let summary = nd2.summary()?;
    let pixels = nd2.read_frame(0)?;
    let expected =
        summary.sizes["Y"] * summary.sizes["X"] * summary.sizes.get("C").copied().unwrap_or(1);
    assert_eq!(pixels.len(), expected);
    Ok(())
}

#[test]
fn test_read_frame_last() -> Result<()> {
    let mut nd2 = match require_fixture() {
        Some(x) => x,
        None => return Ok(()),
    };

    let summary = nd2.summary()?;
    let last = summary.logical_frame_count.saturating_sub(1);
    let pixels = nd2.read_frame(last)?;
    assert!(!pixels.is_empty());
    Ok(())
}

#[test]
fn test_read_frame_2d() -> Result<()> {
    let mut nd2 = match require_fixture() {
        Some(x) => x,
        None => return Ok(()),
    };

    let summary = nd2.summary()?;
    let frame = nd2.read_frame_2d(0, 0, 0, 0)?;
    let expected = summary.sizes["Y"] * summary.sizes["X"];
    assert_eq!(frame.len(), expected);
    Ok(())
}

#[test]
fn test_read_frame_out_of_range() -> Result<()> {
    let mut nd2 = match require_fixture() {
        Some(x) => x,
        None => return Ok(()),
    };

    let summary = nd2.summary()?;
    assert!(nd2.read_frame(summary.logical_frame_count + 1).is_err());
    Ok(())
}

#[test]
fn test_read_frame_2d_out_of_range() -> Result<()> {
    let mut nd2 = match require_fixture() {
        Some(x) => x,
        None => return Ok(()),
    };

    let summary = nd2.summary()?;
    let p = summary.sizes.get("P").copied().unwrap_or(1);
    let t = summary.sizes.get("T").copied().unwrap_or(1);
    let c = summary.sizes.get("C").copied().unwrap_or(1);
    let z = summary.sizes.get("Z").copied().unwrap_or(1);

    assert!(nd2.read_frame_2d(p, 0, 0, 0).is_err());
    assert!(nd2.read_frame_2d(0, t, 0, 0).is_err());
    assert!(nd2.read_frame_2d(0, 0, c, 0).is_err());
    assert!(nd2.read_frame_2d(0, 0, 0, z).is_err());
    Ok(())
}
