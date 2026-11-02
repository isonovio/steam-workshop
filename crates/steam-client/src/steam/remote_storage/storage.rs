use std::marker::PhantomData;
use std::num::NonZeroU64;

use crate::native::remote_storage;

use super::PublishedFile;

pub use remote_storage::MAX_UPLOAD_SIZE;

#[derive(Clone, Copy)]
#[doc = include_str!("../../tests/steam/remote_storage/storage/contracts.md")]
pub struct RemoteStorage<'runtime, 'storage> {
    pub(super) native: remote_storage::RemoteStorage<'runtime>,
    pub(super) brand: PhantomData<fn(&'storage ()) -> &'storage ()>,
}

impl<'runtime> RemoteStorage<'runtime, '_> {
    pub(in crate::steam) fn scope<T>(
        native: remote_storage::RemoteStorage<'runtime>,
        operation: impl for<'storage> FnOnce(RemoteStorage<'runtime, 'storage>) -> T,
    ) -> T {
        operation(RemoteStorage {
            native,
            brand: PhantomData,
        })
    }
}

impl<'runtime, 'storage> RemoteStorage<'runtime, 'storage> {
    pub fn published_file(self, item_id: NonZeroU64) -> PublishedFile<'runtime, 'storage> {
        PublishedFile::new(self, item_id)
    }
}
