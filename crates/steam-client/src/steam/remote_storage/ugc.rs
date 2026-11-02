use std::time::Duration;

use crate::native::remote_storage::ReadAction;

use crate::utils::try_reserve_exact;

use crate::steam::{CallError, Executor, PendingCall};

use super::RemoteStorage;

impl<'runtime> RemoteStorage<'runtime, '_> {
    pub(crate) fn download_content(
        self,
        content_handle: u64,
        size: u32,
        timeout: Duration,
        executor: &mut Executor<'runtime>,
    ) -> Result<Vec<u8>, CallError> {
        let ugc = self
            .native
            .ugc(content_handle)
            .ok_or(CallError::InvalidHandle)?;
        let mut bytes = try_reserve_exact(size as usize).ok_or(CallError::Allocation)?;

        let call = ugc.download().ok_or(CallError::InvalidHandle)?;

        let download = PendingCall { executor, call }.wait(timeout)?;
        if download.handle != content_handle || download.size as u32 != size {
            return Err(CallError::InvalidResult);
        }

        let read = ugc
            .read(&mut bytes, 0, ReadAction::Close)
            .ok_or(CallError::InvalidResult)?;
        if read != download.size {
            return Err(CallError::InvalidResult);
        }
        Ok(bytes)
    }
}
