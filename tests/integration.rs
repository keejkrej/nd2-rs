//! Integration tests. Set ND2_TEST_FILE to run against a real ND2 file.
//! Example: ND2_TEST_FILE=D:\huh7.nd2 cargo test

use nd2_rs::{Nd2File, Result};

fn test_path() -> Option<std::path::PathBuf> {
    std::env::var("ND2_TEST_FILE").ok().map(|s| s.into())
}

#[test]
fn test_sizes_and_loop_indices() -> Result<()> {
    let path = match test_path() {
        Some(p) if p.exists() => p,
        _ => return Ok(()),
    };

    let mut nd2 = Nd2File::open(&path)?;
    let sizes = nd2.sizes()?;
    let loop_indices = nd2.loop_indices()?;

    let n_pos = *sizes.get("P").unwrap_or(&1);
    let n_time = *sizes.get("T").unwrap_or(&1);
    let n_chan = *sizes.get("C").unwrap_or(&1);
    let n_z = *sizes.get("Z").unwrap_or(&1);
    let expected_len = n_pos * n_time * n_chan * n_z;

    assert_eq!(loop_indices.len(), expected_len, "loop_indices length should match sizes product");

    Ok(())
}

#[test]
fn test_read_frame() -> Result<()> {
    let path = match test_path() {
        Some(p) if p.exists() => p,
        _ => return Ok(()),
    };

    let mut nd2 = Nd2File::open(&path)?;
    let attrs = nd2.attributes()?;
    let h = attrs.height_px as usize;
    let w = attrs.width_px.unwrap_or(0) as usize;
    let n_comp = attrs.component_count as usize;
    let expected_pixels = h * w * n_comp;

    let pixels = nd2.read_frame(0)?;
    assert!(
        pixels.len() >= expected_pixels,
        "read_frame(0) should return at least {} pixels, got {}",
        expected_pixels,
        pixels.len()
    );

    Ok(())
}
