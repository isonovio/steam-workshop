use super::*;

fn result(status: i32, size: i32) -> Vec<u8> {
    let mut bytes = vec![0; size_of::<NativeDownloadResult>()];
    let offset = offset_of!(NativeDownloadResult, m_eResult);
    bytes[offset..offset + 4].copy_from_slice(&status.to_ne_bytes());
    let offset = offset_of!(NativeDownloadResult, m_nSizeInBytes);
    bytes[offset..offset + 4].copy_from_slice(&size.to_ne_bytes());
    let offset = offset_of!(NativeDownloadResult, m_hFile);
    bytes[offset..offset + 8].copy_from_slice(&42_u64.to_ne_bytes());
    bytes
}

#[test]
fn decodes_size_and_content_handle() {
    let value = decode(&result(1, 60091)).unwrap();
    assert_eq!(value.handle, 42);
    assert_eq!(value.size, 60091);
}

#[test]
fn rejects_invalid_results() {
    assert!(matches!(decode(&[]), Err(CallError::InvalidResult)));
    assert!(matches!(
        decode(&result(1, -1)),
        Err(CallError::InvalidResult)
    ));
    assert!(matches!(
        decode(&result(15, -1)),
        Err(CallError::Steam(SteamError::AccessDenied))
    ));
}
