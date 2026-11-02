use super::*;

#[test]
fn preserves_details_at_application_boundary() {
    let item = WorkshopItem::from(PublishedFileDetails {
        content_handle: 11,
        id: 12,
        title: "Test 模组".into(),
        creator_app_id: 13,
        consumer_app_id: 14,
        owner_id: 15,
        filename: "module.bson".into(),
        size: 16,
        updated_at: 17,
        visibility: PublishedFileVisibility::Private,
    });
    assert_eq!(
        item,
        WorkshopItem {
            id: 12,
            title: "Test 模组".into(),
            creator_app_id: 13,
            consumer_app_id: 14,
            owner_id: 15,
            filename: "module.bson".into(),
            size: 16,
            updated_at: 17,
            visibility: Visibility::Private,
        }
    );
}

#[test]
fn preserves_all_visibility_variants() {
    for (native, expected) in [
        (PublishedFileVisibility::Public, Visibility::Public),
        (
            PublishedFileVisibility::FriendsOnly,
            Visibility::FriendsOnly,
        ),
        (PublishedFileVisibility::Private, Visibility::Private),
        (PublishedFileVisibility::Unlisted, Visibility::Unlisted),
    ] {
        assert_eq!(Visibility::from(native), expected);
    }
}
