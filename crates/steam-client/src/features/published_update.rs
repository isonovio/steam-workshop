use std::ffi::CStr;
use std::num::NonZeroU64;

use crate::native::remote_storage::Update;
use crate::native::results::CommitResult;

use super::RemoteStorage;
use super::callbacks::PendingCall;

pub(crate) struct PublishedUpdate<'session> {
    storage: RemoteStorage<'session>,
    native: Update<'session>,
}

impl<'session> PublishedUpdate<'session> {
    pub(crate) fn new(storage: RemoteStorage<'session>, item_id: NonZeroU64) -> Option<Self> {
        let native = storage.native.published_file(item_id.get()).update()?;
        Some(Self { storage, native })
    }

    pub(crate) fn set_file(&mut self, filename: &CStr) -> bool {
        self.native.set_file(filename)
    }

    pub(crate) fn commit(self) -> Option<PendingCall<'session, CommitResult>> {
        let call = self.native.commit()?;
        Some(PendingCall {
            runtime: self.storage.runtime,
            call,
        })
    }
}
