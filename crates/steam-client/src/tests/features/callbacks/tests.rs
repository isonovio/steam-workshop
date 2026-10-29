use super::*;

#[test]
#[ignore = "requires Steam running with SteamAppId and SteamGameId set to 286160"]
fn live_legacy_call() {
    let mut session = Session::connect(std::num::NonZeroU32::new(286160).unwrap()).unwrap();
    // The session owns the interface; this request only reads item metadata.
    let handle = unsafe {
        let storage = sys::SteamAPI_SteamRemoteStorage_v016();
        assert!(!storage.is_null());
        sys::SteamAPI_ISteamRemoteStorage_GetPublishedFileDetails(storage, 3826709471, 0)
    };
    let call = PendingCall {
        session: &mut session,
        handle,
        callback_id: sys::RemoteStorageGetPublishedFileDetailsResult_t_k_iCallback as i32,
        result_size: size_of::<sys::RemoteStorageGetPublishedFileDetailsResult_t>() as i32,
        decode: |bytes| {
            // The waiter checked the callback ID and exact native result size.
            let result = unsafe {
                bytes
                    .as_ptr()
                    .cast::<sys::RemoteStorageGetPublishedFileDetailsResult_t>()
                    .read_unaligned()
            };
            match result.m_eResult {
                sys::EResult::k_EResultOK => Ok(result.m_nPublishedFileId),
                error => Err(CallError::Steam(error.into())),
            }
        },
    };
    assert_eq!(call.wait(Duration::from_secs(30)), Ok(3826709471));
}

#[test]
fn completes_after_pending_poll() {
    let mut polls = 0;
    let result = wait_until(Duration::from_secs(1), || {
        polls += 1;
        Ok((polls == 2).then_some(42))
    });
    assert_eq!(result, Ok(42));
}

#[test]
fn timeout_does_not_poll_or_retry() {
    let result = wait_until::<()>(Duration::ZERO, || panic!("polled after deadline"));
    assert_eq!(result, Err(CallError::Timeout));
    assert_eq!(
        wait_until::<()>(Duration::from_millis(1), || Ok(None)),
        Err(CallError::Timeout)
    );
}

#[test]
fn failed_results_are_not_decoded() {
    let decode = |_: &[u8]| -> Result<(), CallError> { panic!("decoded failed result") };
    assert_eq!(
        decode_result(true, true, &[], decode),
        Err(CallError::Transport)
    );
    assert_eq!(
        decode_result(false, false, &[], decode),
        Err(CallError::InvalidResult)
    );
}

#[test]
fn steam_errors_are_preserved() {
    let error = decode_result(true, false, &[], |_| -> Result<(), CallError> {
        Err(CallError::Steam(SteamError::AccessDenied))
    });
    assert_eq!(error, Err(CallError::Steam(SteamError::AccessDenied)));
    assert_eq!(
        wait_until::<()>(Duration::from_secs(1), || Err(CallError::Transport)),
        Err(CallError::Transport)
    );
}
