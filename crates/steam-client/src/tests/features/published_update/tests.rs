use super::*;

fn native_result(status: i32, legal: u8) -> Vec<u8> {
    let mut bytes = vec![0; size_of::<NativeResult>()];
    let offset = offset_of!(NativeResult, m_eResult);
    bytes[offset..offset + 4].copy_from_slice(&status.to_ne_bytes());
    let offset = offset_of!(NativeResult, m_nPublishedFileId);
    bytes[offset..offset + 8].copy_from_slice(&42_u64.to_ne_bytes());
    bytes[offset_of!(NativeResult, m_bUserNeedsToAcceptWorkshopLegalAgreement)] = legal;
    bytes
}

#[test]
fn decodes_success_and_legal_agreement() {
    for legal in [false, true] {
        assert_eq!(
            decode(&native_result(1, u8::from(legal))),
            Ok(CommitResult {
                item_id: 42,
                result: Ok(legal)
            })
        );
    }
}

#[test]
fn distinguishes_confirmed_failure_from_unreadable_result() {
    assert_eq!(
        decode(&native_result(15, 0)),
        Ok(CommitResult {
            item_id: 42,
            result: Err(CallError::Steam(SteamError::AccessDenied))
        })
    );
    assert_eq!(decode(&[]), Err(CallError::InvalidResult));
    assert_eq!(
        decode(&native_result(i32::MAX, 0)),
        Err(CallError::InvalidResult)
    );
    assert_eq!(decode(&native_result(1, 2)), Err(CallError::InvalidResult));
}
