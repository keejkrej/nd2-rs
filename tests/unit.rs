//! Unit tests that do not require an ND2 file.

use nd2::Nd2File;
use std::io::Write;

#[test]
fn test_open_nonexistent_fails() {
    let res = Nd2File::open("nonexistent_file_xyz.nd2");
    assert!(res.is_err());
    let err = match res {
        Ok(_) => unreachable!(),
        Err(err) => err,
    };
    assert!(err.is_file(), "expected file error for missing path");
}

#[test]
fn test_open_invalid_file_fails() {
    let tmp = std::env::temp_dir().join("nd2_rs_test_garbage.nd2");
    let mut f = std::fs::File::create(&tmp).unwrap();
    f.write_all(&[0u8; 200]).unwrap();
    drop(f);
    let err = match Nd2File::open(&tmp) {
        Ok(_) => unreachable!("garbage file should not open successfully"),
        Err(err) => err,
    };
    assert!(
        err.is_file(),
        "invalid file should be classified as file error"
    );
    let _ = std::fs::remove_file(&tmp);
}

#[test]
fn test_open_reader_invalid_stream() {
    use std::io::Cursor;
    let cursor = Cursor::new(vec![0u8; 100]);
    let err = match Nd2File::open_reader(cursor) {
        Ok(_) => unreachable!("empty cursor should fail to parse"),
        Err(err) => err,
    };
    assert!(err.is_file());
    assert_eq!(err.source(), nd2::ErrorSource::File);
}

#[test]
fn test_error_classification_helpers() {
    let err = nd2::Nd2Error::file_invalid_format("test format");
    assert!(err.is_file());
    assert!(!err.is_input());
    assert!(!err.is_internal());
    assert!(!err.is_unsupported());

    let err = nd2::Nd2Error::input_out_of_range("sequence index", 5, 2);
    assert!(!err.is_file());
    assert!(err.is_input());

    let err = nd2::Nd2Error::internal_overflow("sequence stride multiply");
    assert!(err.is_internal());

    let err = nd2::Nd2Error::unsupported_version(99, 0);
    assert!(err.is_unsupported());
}
