use crate::native::{Runtime, remote_storage};

pub use remote_storage::MAX_UPLOAD_SIZE;

#[derive(Clone, Copy)]
pub struct RemoteStorage<'session> {
    pub(super) native: remote_storage::RemoteStorage<'session>,
}

impl<'session> RemoteStorage<'session> {
    pub(crate) fn new(runtime: &'session Runtime) -> Option<Self> {
        Some(Self {
            native: remote_storage::RemoteStorage::new(runtime)?,
        })
    }
}
