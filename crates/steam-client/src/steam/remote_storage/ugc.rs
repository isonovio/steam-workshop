use std::time::Duration;

use crate::native::remote_storage::{self, ReadAction};

use crate::utils::try_reserve_exact;

use crate::steam::{CallError, Executor, PendingCall};

pub(super) struct Ugc<'runtime> {
    native: remote_storage::Ugc<'runtime>,
    size: u32,
}

impl<'runtime> Ugc<'runtime> {
    pub(super) fn new(native: remote_storage::Ugc<'runtime>, size: u32) -> Self {
        Self { native, size }
    }

    pub(super) fn download(
        self,
        timeout: Duration,
        executor: &mut Executor<'runtime>,
    ) -> Result<Vec<u8>, CallError> {
        let mut bytes = try_reserve_exact(self.size as usize).ok_or(CallError::Allocation)?;

        let call = self.native.download().ok_or(CallError::InvalidHandle)?;

        let download = PendingCall { executor, call }.wait(timeout)?;
        if download.handle != self.native.handle() || download.size as u32 != self.size {
            return Err(CallError::InvalidResult);
        }

        let read = self
            .native
            .read(&mut bytes, 0, ReadAction::Close)
            .ok_or(CallError::InvalidResult)?;
        if read != download.size {
            return Err(CallError::InvalidResult);
        }
        Ok(bytes)
    }
}
