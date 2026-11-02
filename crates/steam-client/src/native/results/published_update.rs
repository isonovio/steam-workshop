use std::mem::{offset_of, size_of};

use steamworks::{SteamError, sys};

use crate::utils::native::field;

use super::super::ApiError;

use super::CommitResult;

type NativeResult = sys::RemoteStorageUpdatePublishedFileResult_t;

impl CommitResult {
    pub(in crate::native) fn decode(bytes: &[u8]) -> Result<Self, ApiError> {
        let bytes: &[u8; size_of::<NativeResult>()] =
            bytes.try_into().map_err(|_| ApiError::InvalidResult)?;
        let item_id =
            u64::from_ne_bytes(field(bytes, offset_of!(NativeResult, m_nPublishedFileId)));
        let status = i32::from_ne_bytes(field(bytes, offset_of!(NativeResult, m_eResult)));
        let result = match status == sys::EResult::k_EResultOK as i32 {
            true => Ok(
                match bytes[offset_of!(NativeResult, m_bUserNeedsToAcceptWorkshopLegalAgreement)] {
                    0 => false,
                    1 => true,
                    _ => return Err(ApiError::InvalidResult),
                },
            ),
            false => Err(ApiError::Steam(
                SteamError::try_from(i64::from(status)).map_err(|_| ApiError::InvalidResult)?,
            )),
        };
        Ok(CommitResult { item_id, result })
    }
}

#[cfg(test)]
#[path = "../../tests/native/results/published_update/tests.rs"]
mod tests;
