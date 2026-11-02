use std::mem::size_of;
mod update;

use steamworks::sys;

use crate::native::ApiCall;
use crate::native::results::PublishedFileDetails;

use super::RemoteStorage;

pub(crate) use update::Update;

pub(crate) struct PublishedFile<'runtime> {
    storage: RemoteStorage<'runtime>,
    item_id: u64,
}

impl<'runtime> PublishedFile<'runtime> {
    pub(super) fn new(storage: RemoteStorage<'runtime>, item_id: u64) -> Self {
        Self { storage, item_id }
    }

    pub(crate) fn remote_storage(&self) -> RemoteStorage<'runtime> {
        self.storage
    }

    pub(crate) fn get_details(&self) -> Option<ApiCall<'runtime, PublishedFileDetails>> {
        let handle = unsafe {
            sys::SteamAPI_ISteamRemoteStorage_GetPublishedFileDetails(
                self.storage.pointer.as_ptr(),
                self.item_id,
                0,
            )
        };
        ApiCall::new(
            handle,
            sys::RemoteStorageGetPublishedFileDetailsResult_t_k_iCallback as i32,
            size_of::<sys::RemoteStorageGetPublishedFileDetailsResult_t>(),
            PublishedFileDetails::decode,
        )
    }

    pub(crate) fn update(&self) -> Option<Update<'runtime>> {
        Update::new(self.storage, self.item_id)
    }
}
