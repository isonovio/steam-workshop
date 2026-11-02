mod update;

use steamworks::sys;

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

    pub(crate) fn get_details(self) -> u64 {
        unsafe {
            sys::SteamAPI_ISteamRemoteStorage_GetPublishedFileDetails(
                self.storage.pointer.as_ptr(),
                self.item_id,
                0,
            )
        }
    }

    pub(crate) fn update(self) -> Option<Update<'runtime>> {
        Update::new(self.storage, self.item_id)
    }
}
