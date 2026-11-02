use std::ffi::CStr;

use steamworks::sys;

use super::super::RemoteStorage;

pub(crate) struct Update<'runtime> {
    storage: RemoteStorage<'runtime>,
    handle: sys::PublishedFileUpdateHandle_t,
}

impl<'runtime> Update<'runtime> {
    pub(super) fn new(storage: RemoteStorage<'runtime>, item_id: u64) -> Option<Self> {
        let handle = unsafe {
            sys::SteamAPI_ISteamRemoteStorage_CreatePublishedFileUpdateRequest(
                storage.pointer.as_ptr(),
                item_id,
            )
        };
        (handle != sys::k_PublishedFileUpdateHandleInvalid).then_some(Self { storage, handle })
    }

    pub(crate) fn set_file(&mut self, filename: &CStr) -> bool {
        unsafe {
            sys::SteamAPI_ISteamRemoteStorage_UpdatePublishedFileFile(
                self.storage.pointer.as_ptr(),
                self.handle,
                filename.as_ptr(),
            )
        }
    }

    pub(crate) fn commit(self) -> u64 {
        unsafe {
            sys::SteamAPI_ISteamRemoteStorage_CommitPublishedFileUpdate(
                self.storage.pointer.as_ptr(),
                self.handle,
            )
        }
    }
}
