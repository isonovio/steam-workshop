use std::ffi::CStr;
use std::ptr::NonNull;

use steamworks::sys;

use crate::native::Runtime;

use crate::utils::try_reserve_exact;

pub const MAX_UPLOAD_SIZE: usize = sys::k_unMaxCloudFileChunkSize as usize;

pub struct RemoteStorage<'session> {
    pub(super) pointer: NonNull<sys::ISteamRemoteStorage>,
    pub(super) runtime: &'session mut Runtime,
}

impl<'session> RemoteStorage<'session> {
    pub(crate) fn new(runtime: &'session mut Runtime) -> Option<Self> {
        // The exclusive session borrow keeps the SDK alive for this interface.
        let pointer = unsafe { sys::SteamAPI_SteamRemoteStorage_v016() };
        Some(Self {
            pointer: NonNull::new(pointer)?,
            runtime,
        })
    }

    pub(super) fn reborrow(&mut self) -> RemoteStorage<'_> {
        RemoteStorage {
            pointer: self.pointer,
            runtime: &mut *self.runtime,
        }
    }

    pub(super) fn exists(&self, file: &CStr) -> bool {
        // The interface retains the session and the name is NUL-terminated.
        unsafe { sys::SteamAPI_ISteamRemoteStorage_FileExists(self.this(), file.as_ptr()) }
    }

    pub(super) fn get_size(&self, file: &CStr) -> Option<usize> {
        let size =
            unsafe { sys::SteamAPI_ISteamRemoteStorage_GetFileSize(self.this(), file.as_ptr()) };
        size.try_into().ok()
    }

    pub(super) fn read(&self, file: &CStr) -> Option<Vec<u8>> {
        self.exists(file).then_some(())?;
        let size = self.get_size(file)?;
        let mut buffer = try_reserve_exact(size)?;
        let read = unsafe {
            sys::SteamAPI_ISteamRemoteStorage_FileRead(
                self.this(),
                file.as_ptr(),
                buffer.as_mut_ptr().cast(),
                size as i32,
            )
        };
        (read == size as i32).then_some(buffer)
    }

    pub(super) fn write(&self, file: &CStr, bytes: &[u8]) -> bool {
        unsafe {
            sys::SteamAPI_ISteamRemoteStorage_FileWrite(
                self.this(),
                file.as_ptr(),
                bytes.as_ptr().cast(),
                bytes.len() as i32,
            )
        }
    }

    pub(super) fn delete(&self, file: &CStr) -> bool {
        unsafe { sys::SteamAPI_ISteamRemoteStorage_FileDelete(self.this(), file.as_ptr()) }
    }

    fn this(&self) -> *mut sys::ISteamRemoteStorage {
        self.pointer.as_ptr()
    }
}
