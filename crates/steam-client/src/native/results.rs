mod published_file;
mod published_update;
mod ugc;

use super::ApiError;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PublishedFileDetails {
    pub content_handle: u64,
    pub id: u64,
    pub title: String,
    pub creator_app_id: u32,
    pub consumer_app_id: u32,
    pub owner_id: u64,
    pub filename: String,
    pub size: u32,
    pub updated_at: u32,
    pub visibility: PublishedFileVisibility,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PublishedFileVisibility {
    Public,
    FriendsOnly,
    Private,
    Unlisted,
}

pub(crate) struct DownloadResult {
    pub handle: u64,
    pub size: i32,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CommitResult {
    pub item_id: u64,
    pub result: Result<bool, ApiError>,
}
