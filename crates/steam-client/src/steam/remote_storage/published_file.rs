use std::marker::PhantomData;
use std::num::NonZeroU64;

use crate::native::remote_storage;
use crate::native::results::PublishedFileDetails;
use crate::utils::Timeout;

use crate::steam::{CallError, Executor, PendingCall};

use super::RemoteStorage;
use super::ugc::Ugc;

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

    pub fn download(
        &self,
        executor: &mut Executor<'runtime>,
        timeout: Timeout,
    ) -> Result<Vec<u8>, CallError> {
        let content = self.content(executor, timeout)?;
        content.download(timeout, executor)
    }

    fn content(
        &self,
        executor: &mut Executor<'runtime>,
        timeout: Timeout,
    ) -> Result<Ugc<'runtime>, CallError> {
        let details = self.get_details(timeout, executor)?;
        let storage = self.native.remote_storage();
        let native_ugc = storage
            .ugc(details.content_handle)
            .ok_or(CallError::InvalidHandle)?;
        Ok(Ugc::new(native_ugc, details.size))
    }

    pub(crate) fn get_details(
        &self,
        timeout: Timeout,
        executor: &mut Executor<'runtime>,
    ) -> Result<PublishedFileDetails, CallError> {
        let timeout = timeout.check().ok_or(CallError::Timeout)?;
        let item_id = self.item_id;
        let call = self.native.get_details().ok_or(CallError::InvalidHandle)?;
        let item = PendingCall { executor, call }.wait(timeout)?;
        if item.id != item_id.get() {
            return Err(CallError::InvalidResult);
        }
        Ok(item)
    }
}
