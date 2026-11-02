use std::time::Duration;

use super::*;

#[test]
fn confirmed_commit_allows_cleanup_and_preserves_legal_status() {
    let id = NonZeroU64::new(42).unwrap();
    for legal in [false, true] {
        assert!(
            matches!(completion(Ok(CommitResult { item_id: 42, result: Ok(legal) }), id), Completion::Confirmed(Ok(value)) if value == legal)
        );
    }
    assert!(matches!(
        completion(
            Ok(CommitResult {
                item_id: 42,
                result: Err(crate::native::ApiError::Steam(
                    steamworks::SteamError::AccessDenied
                ))
            }),
            id
        ),
        Completion::Confirmed(Err(CallError::Steam(steamworks::SteamError::AccessDenied)))
    ));
}

#[test]
fn unconfirmed_commit_retains_staging() {
    let id = NonZeroU64::new(42).unwrap();
    for error in [
        CallError::Timeout,
        CallError::Transport,
        CallError::InvalidResult,
    ] {
        assert!(matches!(
            completion(Err(error), id),
            Completion::Uncertain(_)
        ));
    }
    assert!(matches!(
        completion(
            Ok(CommitResult {
                item_id: 43,
                result: Ok(false)
            }),
            id
        ),
        Completion::Uncertain(CallError::InvalidResult)
    ));
}

#[test]
#[ignore = "writes isolated Cloud files; simulates completion without publication; requires app 286160"]
fn live_completion_ownership() {
    use crate::Session;
    use std::ffi::CString;
    use std::num::NonZeroU32;

    let mut session = Session::connect(NonZeroU32::new(286160).unwrap()).unwrap();
    let (steam, _) = session.access();
    steam
        .remote_storage(|storage| {
            let id = NonZeroU64::new(42).unwrap();
            let outcomes = [
                Ok(CommitResult {
                    item_id: 42,
                    result: Ok(true),
                }),
                Ok(CommitResult {
                    item_id: 42,
                    result: Err(crate::native::ApiError::Steam(
                        steamworks::SteamError::AccessDenied,
                    )),
                }),
                Err(CallError::Timeout),
                Err(CallError::Transport),
                Err(CallError::InvalidResult),
                Ok(CommitResult {
                    item_id: 43,
                    result: Ok(false),
                }),
            ];
            for result in outcomes {
                let confirmed = matches!(&result, Ok(value) if value.item_id == 42);
                let name = format!(
                    "steam-workshop-{:032x}.tmp",
                    crate::utils::random_u128().unwrap()
                );
                let file = CloudFile::new(storage, &name, b"ownership").unwrap();
                match resolve(file, result, id) {
                    UpdateOutcome::NotSubmitted { .. } => {
                        panic!("completion cannot undo submission")
                    }
                    UpdateOutcome::Complete { file, .. } => {
                        assert!(confirmed);
                        assert_eq!(file.read().unwrap(), b"ownership");
                        assert!(file.delete().is_ok());
                    }
                    UpdateOutcome::Uncertain { filename, .. } => {
                        assert!(!confirmed);
                        assert_eq!(filename, name);
                        let native = storage.native.file(CString::new(filename).unwrap());
                        assert!(native.exists());
                        assert!(native.delete());
                    }
                }
            }
        })
        .unwrap();
}

#[test]
#[ignore = "writes isolated Cloud files; no submission or callback polling; requires app 286160"]
fn live_pending_drop_and_timeout_retain_content() {
    use crate::Session;
    use std::ffi::CString;
    use std::num::NonZeroU32;

    let mut session = Session::connect(NonZeroU32::new(286160).unwrap()).unwrap();
    let (steam, mut executor) = session.access();
    steam.remote_storage(|storage| {
        for timeout in [false, true] {
            let name = format!("steam-workshop-{:032x}.tmp", crate::utils::random_u128().unwrap());
            let file = CloudFile::new(storage, &name, b"pending").unwrap();
            let pending = PendingUpdate {
                call: PendingCall { executor: &mut executor, call: ApiCall::unpolled_for_test() },
                file,
                item: storage.published_file(NonZeroU64::new(42).unwrap()),
            };
            assert_eq!(pending.file.name(), name);
            match timeout {
                false => drop(pending),
                true => assert!(matches!(pending.wait(Timeout::new(Duration::ZERO)), UpdateOutcome::Uncertain { filename, error: CallError::Timeout } if filename == name)),
            }
            let native = storage.native.file(CString::new(name).unwrap());
            assert!(native.exists());
            assert!(native.delete());
        }
    }).unwrap();
}

#[test]
#[ignore = "writes isolated Cloud files; simulates rejected submission; requires app 286160"]
fn live_rejected_submission_returns_ownership() {
    use crate::Session;
    use std::num::NonZeroU32;

    let mut session = Session::connect(NonZeroU32::new(286160).unwrap()).unwrap();
    let (steam, mut executor) = session.access();
    steam
        .remote_storage(|storage| {
            for error in [CallError::InvalidHandle, CallError::InvalidResult] {
                let name = format!(
                    "steam-workshop-{:032x}.tmp",
                    crate::utils::random_u128().unwrap()
                );
                let file = CloudFile::new(storage, &name, b"rejected").unwrap();
                let rejected = match PendingUpdate::new(
                    storage.published_file(NonZeroU64::new(42).unwrap()),
                    file,
                    &mut executor,
                    Err(error),
                ) {
                    Ok(_) => panic!("rejected call became pending"),
                    Err(rejected) => rejected,
                };
                assert!(matches!(
                    rejected.error,
                    CallError::InvalidHandle | CallError::InvalidResult
                ));
                assert_eq!(rejected.file.read().unwrap(), b"rejected");
                assert!(rejected.file.delete().is_ok());
            }
        })
        .unwrap();
}

#[test]
#[ignore = "writes isolated Cloud content; zero timeout prevents native submission; requires app 286160"]
fn live_zero_timeout_returns_file_without_submission() {
    use crate::Session;
    use std::num::NonZeroU32;

    let mut session = Session::connect(NonZeroU32::new(286160).unwrap()).unwrap();
    let (steam, mut executor) = session.access();
    steam
        .remote_storage(|storage| {
            let name = format!(
                "steam-workshop-{:032x}.tmp",
                crate::utils::random_u128().unwrap()
            );
            let file = CloudFile::new(storage, &name, b"not submitted").unwrap();
            let item = storage.published_file(NonZeroU64::new(3826709471).unwrap());
            match item.update_with(file, &mut executor, Timeout::new(Duration::ZERO)) {
                UpdateOutcome::NotSubmitted {
                    file,
                    error: CallError::Timeout,
                } => {
                    assert_eq!(file.read().unwrap(), b"not submitted");
                    assert!(file.delete().is_ok());
                }
                _ => panic!("expired operation must not be submitted"),
            }
        })
        .unwrap();
}
