use std::ffi::CStr;
use std::mem::{offset_of, size_of};

use steamworks::{SteamError, sys};

use crate::utils::native::field;

use super::super::ApiError;

use super::PublishedFileDetails;
use super::PublishedFileVisibility;

type Details = sys::RemoteStorageGetPublishedFileDetailsResult_t;

impl PublishedFileDetails {
    pub(in crate::native) fn decode(bytes: &[u8]) -> Result<Self, ApiError> {
        let bytes: &[u8; size_of::<Details>()] = match bytes.try_into() {
            Ok(bytes) => bytes,
            Err(_) => return Err(ApiError::InvalidResult),
        };

        let result = i32::from_ne_bytes(field(bytes, offset_of!(Details, m_eResult)));
        if result != sys::EResult::k_EResultOK as i32 {
            return Err(match SteamError::try_from(i64::from(result)) {
                Ok(err) => ApiError::Steam(err),
                Err(_) => ApiError::InvalidResult,
            });
        }

        PublishedFileDetails::from_bytes(bytes).ok_or(ApiError::InvalidResult)
    }
}

impl PublishedFileVisibility {
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

impl PublishedFileDetails {
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
            visibility: PublishedFileVisibility::from_repr(visibility)?,
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
#[path = "../../tests/native/results/published_file/tests.rs"]
mod tests;
