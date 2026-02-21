//! Integration tests for nd2-rs.
//!
//! Without ND2_TEST_FILE: tests skip (pass).
//! With ND2_TEST_FILE pointing to a valid ND2: full validation.
//!
//! Example: ND2_TEST_FILE=D:\huh7.nd2 cargo test
//!
//! CI downloads a small OME fixture automatically.

use nd2_rs::{Nd2File, Result};
use std::path::PathBuf;

fn test_path() -> Option<PathBuf> {
    std::env::var("ND2_TEST_FILE").ok().map(|s| s.into())
}

fn require_fixture() -> Option<(PathBuf, Nd2File)> {
    let path = test_path()?;
    if !path.exists() {
        return None;
    }
    let nd2 = Nd2File::open(&path).ok()?;
    Some((path, nd2))
}

// --- Metadata tests ---

#[test]
fn test_version() -> Result<()> {
    let (_, nd2) = match require_fixture() {
        Some(x) => x,
        None => return Ok(()),
    };

    let v = nd2.version();
    assert!(v.0 >= 2 && v.0 <= 3, "expected modern ND2 version 2.x or 3.x, got {}.{}", v.0, v.1);

    Ok(())
}

#[test]
fn test_attributes() -> Result<()> {
    let (_, mut nd2) = match require_fixture() {
        Some(x) => x,
        None => return Ok(()),
    };

    let attrs = nd2.attributes()?;
    assert!(attrs.height_px > 0, "height_px must be positive");
    assert!(attrs.component_count > 0, "component_count must be positive");
    assert!(attrs.sequence_count > 0, "sequence_count must be positive");
    assert!(attrs.bits_per_component_in_memory > 0, "bits_per_component_in_memory must be positive");

    Ok(())
}

#[test]
fn test_text_info() -> Result<()> {
    let (_, mut nd2) = match require_fixture() {
        Some(x) => x,
        None => return Ok(()),
    };

    let _ = nd2.text_info()?;
    Ok(())
}

#[test]
fn test_experiment() -> Result<()> {
    let (_, mut nd2) = match require_fixture() {
        Some(x) => x,
        None => return Ok(()),
    };

    let _ = nd2.experiment()?;
    Ok(())
}

#[test]
fn test_sizes() -> Result<()> {
    let (_, mut nd2) = match require_fixture() {
        Some(x) => x,
        None => return Ok(()),
    };

    let sizes = nd2.sizes()?;
    assert!(sizes.contains_key("Y"), "sizes must have Y");
    assert!(sizes.contains_key("X"), "sizes must have X");
    let y = *sizes.get("Y").unwrap();
    let x = *sizes.get("X").unwrap();
    assert!(y > 0 && x > 0, "Y and X must be positive");
    let total: usize = sizes
        .get("P").copied().unwrap_or(1)
        * sizes.get("T").copied().unwrap_or(1)
        * sizes.get("C").copied().unwrap_or(1)
        * sizes.get("Z").copied().unwrap_or(1);
    assert!(total > 0, "product of P*T*C*Z must be positive");

    Ok(())
}

#[test]
fn test_loop_indices() -> Result<()> {
    let (_, mut nd2) = match require_fixture() {
        Some(x) => x,
        None => return Ok(()),
    };

    let sizes = nd2.sizes()?;
    let n_pos = *sizes.get("P").unwrap_or(&1);
    let n_time = *sizes.get("T").unwrap_or(&1);
    let n_chan = *sizes.get("C").unwrap_or(&1);
    let n_z = *sizes.get("Z").unwrap_or(&1);
    let expected_len = n_pos * n_time * n_chan * n_z;

    let loop_indices = nd2.loop_indices()?;
    assert_eq!(
        loop_indices.len(),
        expected_len,
        "loop_indices length should match sizes product P*T*C*Z"
    );

    for (seq, m) in loop_indices.iter().enumerate() {
        let p = *m.get("P").unwrap_or(&0);
        let t = *m.get("T").unwrap_or(&0);
        let c = *m.get("C").unwrap_or(&0);
        let z = *m.get("Z").unwrap_or(&0);
        let reconstructed = p * n_time * n_chan * n_z + t * n_chan * n_z + c * n_z + z;
        assert_eq!(
            reconstructed,
            seq,
            "loop_indices[{}] P={} T={} C={} Z={} should reconstruct to seq {}",
            seq, p, t, c, z, seq
        );
    }

    Ok(())
}

#[test]
fn test_chunk_names() -> Result<()> {
    let (_, nd2) = match require_fixture() {
        Some(x) => x,
        None => return Ok(()),
    };

    let chunks = nd2.chunk_names();
    assert!(!chunks.is_empty(), "chunk list must not be empty");
    assert!(
        chunks.iter().any(|c| c.starts_with("ImageDataSeq|")),
        "expected ImageDataSeq| chunks"
    );

    Ok(())
}

// --- YX frame read tests ---

#[test]
fn test_read_frame_0() -> Result<()> {
    let (_, mut nd2) = match require_fixture() {
        Some(x) => x,
        None => return Ok(()),
    };

    let attrs = nd2.attributes()?;
    let h = attrs.height_px as usize;
    let w = attrs.width_px.unwrap_or(0) as usize;
    let n_comp = attrs.component_count as usize;
    let expected_pixels = h * w * n_comp;

    let pixels = nd2.read_frame(0)?;
    assert_eq!(
        pixels.len(),
        expected_pixels,
        "read_frame(0) must return exactly h*w*components = {} pixels",
        expected_pixels
    );

    Ok(())
}

#[test]
fn test_read_frame_yx_shape() -> Result<()> {
    let (_, mut nd2) = match require_fixture() {
        Some(x) => x,
        None => return Ok(()),
    };

    let sizes = nd2.sizes()?;
    let h = *sizes.get("Y").unwrap();
    let w = *sizes.get("X").unwrap();
    let n_c = *sizes.get("C").unwrap_or(&1);
    let n_z = *sizes.get("Z").unwrap_or(&1);
    let total_frames = *sizes.get("P").unwrap_or(&1)
        * sizes.get("T").copied().unwrap_or(1)
        * n_c
        * n_z;

    let pixels = nd2.read_frame(0)?;
    let expected = h * w * n_c;
    assert_eq!(pixels.len(), expected, "frame 0 should be Y*X*C = {} px", expected);

    if total_frames > 1 {
        let last = total_frames - 1;
        let pixels_last = nd2.read_frame(last)?;
        assert_eq!(pixels_last.len(), expected, "frame {} should match shape", last);
    }

    Ok(())
}

#[test]
fn test_read_frame_last() -> Result<()> {
    let (_, mut nd2) = match require_fixture() {
        Some(x) => x,
        None => return Ok(()),
    };

    let loop_indices = nd2.loop_indices()?;
    if loop_indices.is_empty() {
        return Ok(());
    }

    let last_idx = loop_indices.len() - 1;
    let pixels = nd2.read_frame(last_idx)?;
    assert!(!pixels.is_empty(), "read_frame(last) must return non-empty");
    Ok(())
}

#[test]
fn test_read_frame_out_of_range() -> Result<()> {
    let (_, mut nd2) = match require_fixture() {
        Some(x) => x,
        None => return Ok(()),
    };

    let loop_indices = nd2.loop_indices()?;
    let bad_idx = loop_indices.len() + 100;
    let res = nd2.read_frame(bad_idx);
    assert!(res.is_err(), "read_frame out of range must error");
    Ok(())
}
