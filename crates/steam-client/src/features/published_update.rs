use std::ffi::CStr;
use std::num::NonZeroU64;

use crate::native::remote_storage::Update;
use crate::native::results::CommitResult;

use crate::steam::{Executor, PendingCall};

use super::RemoteStorage;

pub(crate) struct PublishedUpdate<'session> {
    native: Update<'session>,
}

impl<'session> PublishedUpdate<'session> {
    pub(crate) fn new(storage: RemoteStorage<'session>, item_id: NonZeroU64) -> Option<Self> {
        let native = storage.native.published_file(item_id.get()).update()?;
        Some(Self { native })
    }

    pub(crate) fn set_file(&mut self, filename: &CStr) -> bool {
        self.native.set_file(filename)
    }

    pub(crate) fn commit<'a>(
        self,
        executor: &'a mut Executor<'session>,
    ) -> Option<PendingCall<'a, 'session, CommitResult>> {
        let call = self.native.commit()?;
        Some(PendingCall { executor, call })
    }
}
