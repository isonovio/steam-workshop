use super::*;

fn details() -> Vec<u8> {
    let mut bytes = vec![0; size_of::<Details>()];
    put(
        &mut bytes,
        offset_of!(Details, m_eResult),
        &1_i32.to_ne_bytes(),
    );
    put(
        &mut bytes,
        offset_of!(Details, m_nPublishedFileId),
        &3826709471_u64.to_ne_bytes(),
    );
    put(
        &mut bytes,
        offset_of!(Details, m_nConsumerAppID),
        &286160_u32.to_ne_bytes(),
    );
    put(
        &mut bytes,
        offset_of!(Details, m_rgchTitle),
        "Test 模组\0".as_bytes(),
    );
    put(
        &mut bytes,
        offset_of!(Details, m_eVisibility),
        &2_u32.to_ne_bytes(),
    );
    bytes
}

fn put(bytes: &mut [u8], offset: usize, value: &[u8]) {
    bytes[offset..offset + value.len()].copy_from_slice(value);
}

#[test]
fn decodes_native_layout_and_unicode() {
    let item = decode(&details()).unwrap();
    assert_eq!(item.id, 3826709471);
    assert_eq!(item.consumer_app_id, 286160);
    assert_eq!(item.title, "Test 模组");
    assert_eq!(item.visibility, Visibility::Private);
}

#[test]
fn preserves_steam_failure_before_reading_metadata() {
    let mut bytes = details();
    put(
        &mut bytes,
        offset_of!(Details, m_eResult),
        &(sys::EResult::k_EResultAccessDenied as i32).to_ne_bytes(),
    );
    put(
        &mut bytes,
        offset_of!(Details, m_eVisibility),
        &u32::MAX.to_ne_bytes(),
    );
    assert_eq!(
        decode(&bytes),
        Err(CallError::Steam(SteamError::AccessDenied))
    );
}

#[test]
fn rejects_invalid_layout_and_values() {
    assert_eq!(decode(&[]), Err(CallError::InvalidResult));
    let mut bytes = details();
    bytes.pop();
    assert_eq!(decode(&bytes), Err(CallError::InvalidResult));
    let mut bytes = details();
    put(
        &mut bytes,
        offset_of!(Details, m_nFileSize),
        &(-1_i32).to_ne_bytes(),
    );
    assert_eq!(decode(&bytes), Err(CallError::InvalidResult));
    let mut bytes = details();
    put(
        &mut bytes,
        offset_of!(Details, m_eVisibility),
        &u32::MAX.to_ne_bytes(),
    );
    assert_eq!(decode(&bytes), Err(CallError::InvalidResult));
}

#[test]
fn rejects_unterminated_or_invalid_utf8_strings() {
    let mut bytes = details();
    let offset = offset_of!(Details, m_rgchTitle);
    bytes[offset..offset + 129].fill(b'x');
    assert_eq!(decode(&bytes), Err(CallError::InvalidResult));
    let mut bytes = details();
    bytes[offset] = 0xff;
    assert_eq!(decode(&bytes), Err(CallError::InvalidResult));
}
