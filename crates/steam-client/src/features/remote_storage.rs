use crate::native::{Runtime, remote_storage};

pub use remote_storage::MAX_UPLOAD_SIZE;

pub struct RemoteStorage<'session> {
    pub(super) native: remote_storage::RemoteStorage<'session>,
    pub(super) runtime: &'session mut Runtime,
}

impl<'session> RemoteStorage<'session> {
    pub(crate) fn new(runtime: &'session mut Runtime) -> Option<Self> {
        // This owner and its derived operations retain the runtime borrow.
        let native = unsafe { remote_storage::RemoteStorage::new()? };
        Some(Self { native, runtime })
    }

    pub(super) fn reborrow(&mut self) -> RemoteStorage<'_> {
        RemoteStorage {
            native: self.native,
            runtime: &mut *self.runtime,
        }
    }
}
