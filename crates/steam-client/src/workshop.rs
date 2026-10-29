use std::ffi::CStr;
use std::mem::{offset_of, size_of};
use std::num::NonZeroU64;
use std::time::Duration;

use steamworks::{SteamError, sys};

use crate::callbacks::{CallError, PendingCall};
use crate::native::field;
use crate::{Session, Visibility, WorkshopItem};

type Details = sys::RemoteStorageGetPublishedFileDetailsResult_t;

impl Session {
    pub fn inspect(
        &mut self,
        item_id: NonZeroU64,
        timeout: Duration,
    ) -> Result<WorkshopItem, CallError> {
        // The session owns the native interface for the duration of this call.
        let handle = unsafe {
            let storage = sys::SteamAPI_SteamRemoteStorage_v016();
            if storage.is_null() {
                return Err(CallError::InvalidResult);
            }
            sys::SteamAPI_ISteamRemoteStorage_GetPublishedFileDetails(storage, item_id.get(), 0)
        };
        let item = PendingCall {
            session: self,
            handle,
            callback_id: sys::RemoteStorageGetPublishedFileDetailsResult_t_k_iCallback as i32,
            result_size: size_of::<Details>() as i32,
            decode,
        }
        .wait(timeout)?;
        if item.id != item_id.get() {
            return Err(CallError::InvalidResult);
        }
        Ok(item)
    }
}

fn decode(bytes: &[u8]) -> Result<WorkshopItem, CallError> {
    let bytes: &[u8; size_of::<Details>()] = match bytes.try_into() {
        Ok(bytes) => bytes,
        Err(_) => return Err(CallError::InvalidResult),
    };

    let result = i32::from_ne_bytes(field(bytes, offset_of!(Details, m_eResult)));
    if result != sys::EResult::k_EResultOK as i32 {
        return Err(match SteamError::try_from(i64::from(result)) {
            Ok(err) => CallError::Steam(err),
            Err(_) => CallError::InvalidResult,
        });
    }

    WorkshopItem::from_bytes(bytes).ok_or(CallError::InvalidResult)
}

impl Visibility {
    pub(crate) fn from_repr(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Public),
            1 => Some(Self::FriendsOnly),
            2 => Some(Self::Private),
            3 => Some(Self::Unlisted),
            _ => None,
        }
    }
}

impl WorkshopItem {
    pub(crate) fn from_bytes(bytes: &[u8; size_of::<Details>()]) -> Option<Self> {
        // Read primitive fields without constructing native enums or packed references.
        let size = i32::from_ne_bytes(field(bytes, offset_of!(Details, m_nFileSize)));
        let visibility = u32::from_ne_bytes(field(bytes, offset_of!(Details, m_eVisibility)));
        Some(Self {
            content_handle: u64::from_ne_bytes(field(bytes, offset_of!(Details, m_hFile))),
            id: u64::from_ne_bytes(field(bytes, offset_of!(Details, m_nPublishedFileId))),
            title: text(bytes, offset_of!(Details, m_rgchTitle), 129)?,
            creator_app_id: u32::from_ne_bytes(field(bytes, offset_of!(Details, m_nCreatorAppID))),
            consumer_app_id: u32::from_ne_bytes(field(
                bytes,
                offset_of!(Details, m_nConsumerAppID),
            )),
            owner_id: u64::from_ne_bytes(field(bytes, offset_of!(Details, m_ulSteamIDOwner))),
            filename: text(bytes, offset_of!(Details, m_pchFileName), 260)?,
            size: size.try_into().ok()?,
            updated_at: u32::from_ne_bytes(field(bytes, offset_of!(Details, m_rtimeUpdated))),
            visibility: Visibility::from_repr(visibility)?,
        })
    }
}

fn text(bytes: &[u8], offset: usize, size: usize) -> Option<String> {
    CStr::from_bytes_until_nul(&bytes[offset..offset + size])
        .ok()?
        .to_str()
        .ok()
        .map(str::to_owned)
}

#[cfg(test)]
#[path = "tests/workshop/tests.rs"]
mod tests;
