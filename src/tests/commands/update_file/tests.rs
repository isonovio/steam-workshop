use super::*;

#[test]
fn reads_file_and_rejects_invalid_inputs() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("test payload.bson");
    assert_eq!(
        read_input(&path).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
    assert_eq!(
        read_input(directory.path()).unwrap_err().kind(),
        io::ErrorKind::InvalidInput
    );
    std::fs::write(&path, b"payload").unwrap();
    assert_eq!(read_input(&path).unwrap(), b"payload");
    File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(MAX_UPLOAD_SIZE as u64 + 1)
        .unwrap();
    assert_eq!(
        read_input(&path).unwrap_err().kind(),
        io::ErrorKind::InvalidInput
    );
}
