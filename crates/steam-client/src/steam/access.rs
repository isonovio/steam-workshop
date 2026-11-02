use super::RemoteStorage;
use crate::native::{Runtime, remote_storage};

#[derive(Clone, Copy)]
pub struct Access<'runtime> {
    runtime: &'runtime Runtime,
}

impl<'runtime> Access<'runtime> {
    pub(super) fn new(runtime: &'runtime Runtime) -> Self {
        Self { runtime }
    }

    pub fn remote_storage<T>(
        self,
        operation: impl for<'storage> FnOnce(RemoteStorage<'runtime, 'storage>) -> T,
    ) -> Option<T> {
        let native = remote_storage::RemoteStorage::new(self.runtime)?;
        Some(RemoteStorage::scope(native, operation))
    }
}
