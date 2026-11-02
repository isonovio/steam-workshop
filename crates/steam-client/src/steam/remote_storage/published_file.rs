use std::marker::PhantomData;
use std::num::NonZeroU64;
use std::time::Duration;

use crate::native::remote_storage;
use crate::native::results::PublishedFileDetails;

use crate::steam::{CallError, Executor, PendingCall};

use super::RemoteStorage;

pub struct PublishedFile<'runtime, 'storage> {
    pub(super) native: remote_storage::PublishedFile<'runtime>,
    pub(super) brand: PhantomData<fn(&'storage ()) -> &'storage ()>,
    item_id: NonZeroU64,
}

impl<'runtime, 'storage> PublishedFile<'runtime, 'storage> {
    pub(super) fn new(storage: RemoteStorage<'runtime, 'storage>, item_id: NonZeroU64) -> Self {
        Self {
            native: storage.native.published_file(item_id.get()),
            brand: storage.brand,
            item_id,
        }
    }

    pub fn item_id(&self) -> NonZeroU64 {
        self.item_id
    }

    pub(crate) fn get_details(
        self,
        timeout: Duration,
        executor: &mut Executor<'runtime>,
    ) -> Result<PublishedFileDetails, CallError> {
        let item_id = self.item_id;
        let call = self.native.get_details().ok_or(CallError::InvalidHandle)?;
        let item = PendingCall { executor, call }.wait(timeout)?;
        if item.id != item_id.get() {
            return Err(CallError::InvalidResult);
        }
        Ok(item)
    }
}
