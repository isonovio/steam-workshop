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
