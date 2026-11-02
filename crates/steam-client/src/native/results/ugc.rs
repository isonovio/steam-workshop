use std::mem::{offset_of, size_of};

use steamworks::{SteamError, sys};

use crate::utils::native::field;

use super::super::ApiError;

use super::DownloadResult;

type NativeDownloadResult = sys::RemoteStorageDownloadUGCResult_t;

impl DownloadResult {
    pub(in crate::native) fn decode(bytes: &[u8]) -> Result<Self, ApiError> {
        let bytes: &[u8; size_of::<NativeDownloadResult>()] = match bytes.try_into() {
            Ok(bytes) => bytes,
            Err(_) => return Err(ApiError::InvalidResult),
        };

        let result = i32::from_ne_bytes(field(bytes, offset_of!(NativeDownloadResult, m_eResult)));
        if result != sys::EResult::k_EResultOK as i32 {
            return Err(match SteamError::try_from(i64::from(result)) {
                Ok(error) => ApiError::Steam(error),
                Err(_) => ApiError::InvalidResult,
            });
        }

        DownloadResult::from_bytes(bytes).ok_or(ApiError::InvalidResult)
    }
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
#[path = "../../tests/native/results/ugc/tests.rs"]
mod tests;
