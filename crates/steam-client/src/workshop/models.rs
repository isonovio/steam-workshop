#[derive(Debug, PartialEq, Eq)]
pub struct WorkshopItem {
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

#[derive(Debug, PartialEq, Eq)]
pub struct UpdateResult {
    pub item_id: u64,
    pub needs_legal_agreement: bool,
}
