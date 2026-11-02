use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr::NonNull;

use steamworks::sys;

use super::super::Runtime;

use super::{File, PublishedFile, Ugc};

pub const MAX_UPLOAD_SIZE: usize = sys::k_unMaxCloudFileChunkSize as usize;

#[derive(Clone, Copy)]
pub(crate) struct RemoteStorage<'runtime> {
    pub(super) pointer: NonNull<sys::ISteamRemoteStorage>,
    _runtime: PhantomData<&'runtime Runtime>,
}

impl<'runtime> RemoteStorage<'runtime> {
    pub(crate) fn new(_runtime: &'runtime Runtime) -> Option<Self> {
        let pointer = unsafe { sys::SteamAPI_SteamRemoteStorage_v016() };
        Some(Self {
            pointer: NonNull::new(pointer)?,
            _runtime: PhantomData,
        })
    }

    pub(crate) fn file(self, name: &CStr) -> File<'runtime, '_> {
        File::new(self, name)
    }

    pub(crate) fn published_file(self, item_id: u64) -> PublishedFile<'runtime> {
        PublishedFile::new(self, item_id)
    }

    pub(crate) fn ugc(self, handle: u64) -> Ugc<'runtime> {
        Ugc::new(self, handle)
    }
}
