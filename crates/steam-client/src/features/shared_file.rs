use std::mem::{offset_of, size_of};
use std::time::Duration;

use steamworks::{SteamError, sys};

use crate::utils::{native::field, try_reserve_exact};

use super::RemoteStorage;
use super::callbacks::{CallError, PendingCall};
use super::results::DownloadResult;

type NativeDownloadResult = sys::RemoteStorageDownloadUGCResult_t;

impl RemoteStorage<'_> {
    pub(crate) fn download_content(
        self,
        content_handle: u64,
        size: u32,
        timeout: Duration,
    ) -> Result<Vec<u8>, CallError> {
        if content_handle == 0 || content_handle == sys::k_UGCHandleInvalid {
            return Err(CallError::InvalidHandle);
        }

        let mut bytes = try_reserve_exact(size as usize).ok_or(CallError::Allocation)?;

        let storage = self.pointer.as_ptr();
        let handle =
            unsafe { sys::SteamAPI_ISteamRemoteStorage_UGCDownload(storage, content_handle, 0) };

        let download = PendingCall {
            session: &mut *self.session,
            handle,
            callback_id: sys::RemoteStorageDownloadUGCResult_t_k_iCallback as i32,
            result_size: size_of::<NativeDownloadResult>() as i32,
            decode,
        }
        .wait(timeout)?;
        if download.handle != content_handle || download.size as u32 != size {
            return Err(CallError::InvalidResult);
        }

        // Buffer length matches the validated native size; Close releases the read handle.
        let read = unsafe {
            sys::SteamAPI_ISteamRemoteStorage_UGCRead(
                storage,
                download.handle,
                bytes.as_mut_ptr().cast(),
                download.size,
                0,
                sys::EUGCReadAction::k_EUGCRead_Close,
            )
        };
        if read != download.size {
            return Err(CallError::InvalidResult);
        }
        Ok(bytes)
    }
}

fn decode(bytes: &[u8]) -> Result<DownloadResult, CallError> {
    let bytes: &[u8; size_of::<NativeDownloadResult>()] = match bytes.try_into() {
        Ok(bytes) => bytes,
        Err(_) => return Err(CallError::InvalidResult),
    };

    let result = i32::from_ne_bytes(field(bytes, offset_of!(NativeDownloadResult, m_eResult)));
    if result != sys::EResult::k_EResultOK as i32 {
        return Err(match SteamError::try_from(i64::from(result)) {
            Ok(error) => CallError::Steam(error),
            Err(_) => CallError::InvalidResult,
        });
    }

    DownloadResult::from_bytes(bytes).ok_or(CallError::InvalidResult)
}

impl DownloadResult {
    fn from_bytes(bytes: &[u8; size_of::<NativeDownloadResult>()]) -> Option<Self> {
        let size = i32::from_ne_bytes(field(
            bytes,
            offset_of!(NativeDownloadResult, m_nSizeInBytes),
        ));
        if size < 0 {
            return None;
        }
        Some(Self {
            handle: u64::from_ne_bytes(field(bytes, offset_of!(NativeDownloadResult, m_hFile))),
            size,
        })
    }
}

#[cfg(test)]
#[path = "../tests/features/shared_file/tests.rs"]
mod tests;
