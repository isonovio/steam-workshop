use std::ffi::CStr;
use std::mem::{offset_of, size_of};
use std::num::NonZeroU64;

use steamworks::{SteamError, sys};

use crate::utils::native::field;

use super::RemoteStorage;
use super::callbacks::{CallError, PendingCall};
use super::results::CommitResult;

type NativeResult = sys::RemoteStorageUpdatePublishedFileResult_t;

pub(crate) struct PublishedUpdate<'session> {
    storage: RemoteStorage<'session>,
    handle: sys::PublishedFileUpdateHandle_t,
}

impl<'session> PublishedUpdate<'session> {
    pub(crate) fn new(storage: RemoteStorage<'session>, item_id: NonZeroU64) -> Option<Self> {
        let handle = unsafe {
            sys::SteamAPI_ISteamRemoteStorage_CreatePublishedFileUpdateRequest(
                storage.pointer.as_ptr(),
                item_id.get(),
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

    pub(crate) fn commit(self) -> Option<PendingCall<'session, CommitResult>> {
        let handle = unsafe {
            sys::SteamAPI_ISteamRemoteStorage_CommitPublishedFileUpdate(
                self.storage.pointer.as_ptr(),
                self.handle,
            )
        };
        (handle != 0).then_some(PendingCall {
            runtime: self.storage.runtime,
            handle,
            callback_id: sys::RemoteStorageUpdatePublishedFileResult_t_k_iCallback as i32,
            result_size: size_of::<NativeResult>() as i32,
            decode,
        })
    }
}

fn decode(bytes: &[u8]) -> Result<CommitResult, CallError> {
    let bytes: &[u8; size_of::<NativeResult>()] =
        bytes.try_into().map_err(|_| CallError::InvalidResult)?;
    let item_id = u64::from_ne_bytes(field(bytes, offset_of!(NativeResult, m_nPublishedFileId)));
    let status = i32::from_ne_bytes(field(bytes, offset_of!(NativeResult, m_eResult)));
    let result = match status == sys::EResult::k_EResultOK as i32 {
        true => Ok(
            match bytes[offset_of!(NativeResult, m_bUserNeedsToAcceptWorkshopLegalAgreement)] {
                0 => false,
                1 => true,
                _ => return Err(CallError::InvalidResult),
            },
        ),
        false => Err(CallError::Steam(
            SteamError::try_from(i64::from(status)).map_err(|_| CallError::InvalidResult)?,
        )),
    };
    Ok(CommitResult { item_id, result })
}

#[cfg(test)]
#[path = "../tests/features/published_update/tests.rs"]
mod tests;
