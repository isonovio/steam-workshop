use steamworks::sys;

use super::RemoteStorage;

pub(crate) struct Ugc<'runtime> {
    storage: RemoteStorage<'runtime>,
    handle: u64,
}

impl<'runtime> Ugc<'runtime> {
    pub(super) fn new(storage: RemoteStorage<'runtime>, handle: u64) -> Self {
        Self { storage, handle }
    }

    pub(crate) fn download(&self) -> u64 {
        unsafe {
            sys::SteamAPI_ISteamRemoteStorage_UGCDownload(
                self.storage.pointer.as_ptr(),
                self.handle,
                0,
            )
        }
    }

    pub(crate) fn read(
        &self,
        buffer: &mut [u8],
        offset: u32,
        action: sys::EUGCReadAction,
    ) -> Option<i32> {
        let size = buffer.len().try_into().ok()?;
        Some(unsafe {
            sys::SteamAPI_ISteamRemoteStorage_UGCRead(
                self.storage.pointer.as_ptr(),
                self.handle,
                buffer.as_mut_ptr().cast(),
                size,
                offset,
                action,
            )
        })
    }
}
