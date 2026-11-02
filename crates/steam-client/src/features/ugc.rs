use std::time::Duration;
use steamworks::sys;

use crate::utils::try_reserve_exact;

use crate::steam::{CallError, Executor, PendingCall};

use super::RemoteStorage;

impl<'runtime> RemoteStorage<'runtime> {
    pub(crate) fn download_content(
        self,
        content_handle: u64,
        size: u32,
        timeout: Duration,
        executor: &mut Executor<'runtime>,
    ) -> Result<Vec<u8>, CallError> {
        if content_handle == 0 || content_handle == sys::k_UGCHandleInvalid {
            return Err(CallError::InvalidHandle);
        }

        let mut bytes = try_reserve_exact(size as usize).ok_or(CallError::Allocation)?;

        let ugc = self.native.ugc(content_handle);
        let call = ugc.download().ok_or(CallError::InvalidHandle)?;

        let download = PendingCall { executor, call }.wait(timeout)?;
        if download.handle != content_handle || download.size as u32 != size {
            return Err(CallError::InvalidResult);
        }

        let read = ugc
            .read(&mut bytes, 0, sys::EUGCReadAction::k_EUGCRead_Close)
            .ok_or(CallError::InvalidResult)?;
        if read != download.size {
            return Err(CallError::InvalidResult);
        }
        Ok(bytes)
    }
}
