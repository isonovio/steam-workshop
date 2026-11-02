use crate::Visibility;

use super::*;

#[test]
fn checks_consumer_app_and_owner() {
    let item = WorkshopItem {
        content_handle: 0,
        id: 1,
        title: String::new(),
        creator_app_id: 99,
        consumer_app_id: 286160,
        owner_id: 42,
        filename: String::new(),
        size: 0,
        updated_at: 0,
        visibility: Visibility::Private,
    };
    assert_eq!(item.validate_update(286160, 42), Ok(()));
    assert_eq!(
        item.validate_update(480, 42),
        Err(UpdateError::WrongApp {
            expected: 480,
            actual: 286160
        })
    );
    assert_eq!(item.validate_update(286160, 43), Err(UpdateError::NotOwner));
}

#[test]
fn confirmed_commit_allows_cleanup_and_preserves_legal_status() {
    let id = NonZeroU64::new(42).unwrap();
    for legal in [false, true] {
        assert!(
            matches!(completion(Ok(CommitResult { item_id: 42, result: Ok(legal) }), id), CloudOperation::Complete(Ok(value)) if value == legal)
        );
    }
    assert!(matches!(
        completion(
            Ok(CommitResult {
                item_id: 42,
                result: Err(CallError::Steam(steamworks::SteamError::AccessDenied))
            }),
            id
        ),
        CloudOperation::Complete(Err(CallError::Steam(steamworks::SteamError::AccessDenied)))
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
            CloudOperation::Uncertain(_)
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
        CloudOperation::Uncertain(CallError::InvalidResult)
    ));
}
