use std::num::NonZeroU64;
use std::time::Duration;

use crate::native::results::PublishedFileDetails;

use crate::steam::{CallError, Executor, PendingCall};

use super::RemoteStorage;

impl<'runtime> RemoteStorage<'runtime> {
    pub(crate) fn published_file_details(
        self,
        item_id: NonZeroU64,
        timeout: Duration,
        executor: &mut Executor<'runtime>,
    ) -> Result<PublishedFileDetails, CallError> {
        let call = self
            .native
            .published_file(item_id.get())
            .get_details()
            .ok_or(CallError::InvalidHandle)?;
        let item = PendingCall { executor, call }.wait(timeout)?;
        if item.id != item_id.get() {
            return Err(CallError::InvalidResult);
        }
        Ok(item)
    }
}
