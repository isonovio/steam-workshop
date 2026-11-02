use crate::Session;

use super::*;

#[test]
#[ignore = "requires Steam running with SteamAppId and SteamGameId set to 286160"]
fn live_legacy_call() {
    let mut session = Session::connect(std::num::NonZeroU32::new(286160).unwrap()).unwrap();
    let storage = session.remote_storage().unwrap();
    let call = storage
        .native
        .published_file(3826709471)
        .get_details()
        .unwrap();
    let result = PendingCall {
        runtime: storage.runtime,
        call,
    }
    .wait(Duration::from_secs(30))
    .unwrap();
    assert_eq!(result.id, 3826709471);
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
fn native_errors_are_preserved() {
    assert_eq!(CallError::from(ApiError::Transport), CallError::Transport);
    assert_eq!(
        CallError::from(ApiError::InvalidResult),
        CallError::InvalidResult
    );
    assert_eq!(
        CallError::from(ApiError::Steam(SteamError::AccessDenied)),
        CallError::Steam(SteamError::AccessDenied)
    );
    assert_eq!(
        wait_until::<()>(Duration::from_secs(1), || Err(CallError::Transport)),
        Err(CallError::Transport)
    );
}
