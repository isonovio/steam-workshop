#[derive(Debug, PartialEq, Eq)]
pub struct WorkshopItem {
    pub(crate) content_handle: u64,
    pub id: u64,
    pub title: String,
    pub creator_app_id: u32,
    pub consumer_app_id: u32,
    pub owner_id: u64,
    pub filename: String,
    pub size: u32,
    pub updated_at: u32,
    pub visibility: Visibility,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Visibility {
    Public,
    FriendsOnly,
    Private,
    Unlisted,
}
