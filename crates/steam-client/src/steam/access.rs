use crate::features::RemoteStorage;
use crate::native::Runtime;

#[derive(Clone, Copy)]
pub struct Access<'runtime> {
    runtime: &'runtime Runtime,
}

impl<'runtime> Access<'runtime> {
    pub(super) fn new(runtime: &'runtime Runtime) -> Self {
        Self { runtime }
    }

    pub fn remote_storage(self) -> Option<RemoteStorage<'runtime>> {
        RemoteStorage::new(self.runtime)
    }
}
