//! Unit tests that do not require an ND2 file.

use nd2_rs::Nd2File;
use std::io::Write;

#[test]
fn test_open_nonexistent_fails() {
    let res = Nd2File::open("nonexistent_file_xyz.nd2");
    assert!(res.is_err());
}

#[test]
fn test_open_invalid_file_fails() {
    let tmp = std::env::temp_dir().join("nd2_rs_test_garbage.nd2");
    let mut f = std::fs::File::create(&tmp).unwrap();
    f.write_all(&[0u8; 200]).unwrap();
    drop(f);
    let res = Nd2File::open(&tmp);
    assert!(res.is_err());
    let _ = std::fs::remove_file(&tmp);
}
