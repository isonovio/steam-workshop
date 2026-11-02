use std::mem::size_of;
use steamworks::sys;

use crate::native::ApiCall;
use crate::native::results::DownloadResult;

use super::RemoteStorage;

pub(crate) enum ReadAction {
    Close,
}

impl From<ReadAction> for sys::EUGCReadAction {
    fn from(action: ReadAction) -> Self {
        match action {
            ReadAction::Close => Self::k_EUGCRead_Close,
        }
    }
}

pub(crate) struct Ugc<'runtime> {
    storage: RemoteStorage<'runtime>,
    handle: u64,
}

impl<'runtime> Ugc<'runtime> {
    pub(super) fn new(storage: RemoteStorage<'runtime>, handle: u64) -> Option<Self> {
        if handle == 0 || handle == sys::k_UGCHandleInvalid {
            return None;
        }
        Some(Self { storage, handle })
    }

    pub(crate) fn handle(&self) -> u64 {
        self.handle
    }

    pub(crate) fn download(&self) -> Option<ApiCall<'runtime, DownloadResult>> {
        let handle = unsafe {
            sys::SteamAPI_ISteamRemoteStorage_UGCDownload(
                self.storage.pointer.as_ptr(),
                self.handle,
                0,
            )
        };
        ApiCall::new(
            handle,
            sys::RemoteStorageDownloadUGCResult_t_k_iCallback as i32,
            size_of::<sys::RemoteStorageDownloadUGCResult_t>(),
            DownloadResult::decode,
        )
    }

    pub(crate) fn read(&self, buffer: &mut [u8], offset: u32, action: ReadAction) -> Option<i32> {
        let size = buffer.len().try_into().ok()?;
        Some(unsafe {
            sys::SteamAPI_ISteamRemoteStorage_UGCRead(
                self.storage.pointer.as_ptr(),
                self.handle,
                buffer.as_mut_ptr().cast(),
                size,
                offset,
                action.into(),
            )
        })
    }
}
