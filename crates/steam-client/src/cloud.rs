use std::mem::{offset_of, size_of};
use std::num::NonZeroU64;
use std::time::{Duration, Instant};

use steamworks::{SteamError, sys};

use crate::Session;
use crate::callbacks::{CallError, PendingCall};
use crate::utils::native::field;

type DownloadResult = sys::RemoteStorageDownloadUGCResult_t;

struct Download {
    handle: u64,
    size: i32,
}

impl Session {
    pub fn download(
        &mut self,
        item_id: NonZeroU64,
        timeout: Duration,
    ) -> Result<Vec<u8>, CallError> {
        let start = Instant::now();

        let item = self.inspect(item_id, timeout)?;
        if item.content_handle == 0 || item.content_handle == sys::k_UGCHandleInvalid {
            return Err(CallError::InvalidHandle);
        }

        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(item.size as usize)
            .map_err(|_| CallError::Allocation)?;
        bytes.resize(item.size as usize, 0);

        let remaining = timeout.saturating_sub(start.elapsed());
        if remaining.is_zero() {
            return Err(CallError::Timeout);
        }

        // The session keeps storage alive through download completion and reading.
        let storage = unsafe { sys::SteamAPI_SteamRemoteStorage_v016() };
        if storage.is_null() {
            return Err(CallError::InvalidResult);
        }

        let handle = unsafe {
            sys::SteamAPI_ISteamRemoteStorage_UGCDownload(storage, item.content_handle, 0)
        };

        let download = PendingCall {
            session: self,
            handle,
            callback_id: sys::RemoteStorageDownloadUGCResult_t_k_iCallback as i32,
            result_size: size_of::<DownloadResult>() as i32,
            decode,
        }
        .wait(remaining)?;
        if download.handle != item.content_handle || download.size as u32 != item.size {
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

fn decode(bytes: &[u8]) -> Result<Download, CallError> {
    let bytes: &[u8; size_of::<DownloadResult>()] = match bytes.try_into() {
        Ok(bytes) => bytes,
        Err(_) => return Err(CallError::InvalidResult),
    };

    let result = i32::from_ne_bytes(field(bytes, offset_of!(DownloadResult, m_eResult)));
    if result != sys::EResult::k_EResultOK as i32 {
        return Err(match SteamError::try_from(i64::from(result)) {
            Ok(error) => CallError::Steam(error),
            Err(_) => CallError::InvalidResult,
        });
    }

    Download::from_bytes(bytes).ok_or(CallError::InvalidResult)
}

impl Download {
    fn from_bytes(bytes: &[u8; size_of::<DownloadResult>()]) -> Option<Self> {
        let size = i32::from_ne_bytes(field(bytes, offset_of!(DownloadResult, m_nSizeInBytes)));
        if size < 0 {
            return None;
        }
        Some(Self {
            handle: u64::from_ne_bytes(field(bytes, offset_of!(DownloadResult, m_hFile))),
            size,
        })
    }
}

#[cfg(test)]
#[path = "tests/cloud/tests.rs"]
mod tests;
