use std::ffi::CStr;

use steamworks::sys;

use super::RemoteStorage;

pub(crate) struct File<'runtime, 'name> {
    storage: RemoteStorage<'runtime>,
    name: &'name CStr,
}

impl<'runtime, 'name> File<'runtime, 'name> {
    pub(super) fn new(storage: RemoteStorage<'runtime>, name: &'name CStr) -> Self {
        Self { storage, name }
    }

    pub(crate) fn exists(&self) -> bool {
        unsafe {
            sys::SteamAPI_ISteamRemoteStorage_FileExists(
                self.storage.pointer.as_ptr(),
                self.name.as_ptr(),
            )
        }
    }

    pub(crate) fn get_size(&self) -> i32 {
        unsafe {
            sys::SteamAPI_ISteamRemoteStorage_GetFileSize(
                self.storage.pointer.as_ptr(),
                self.name.as_ptr(),
            )
        }
    }

    pub(crate) fn read(&self, buffer: &mut [u8]) -> Option<i32> {
        let size = buffer.len().try_into().ok()?;
        Some(unsafe {
            sys::SteamAPI_ISteamRemoteStorage_FileRead(
                self.storage.pointer.as_ptr(),
                self.name.as_ptr(),
                buffer.as_mut_ptr().cast(),
                size,
            )
        })
    }

    pub(crate) fn write(&self, bytes: &[u8]) -> bool {
        let Ok(size) = bytes.len().try_into() else {
            return false;
        };
        unsafe {
            sys::SteamAPI_ISteamRemoteStorage_FileWrite(
                self.storage.pointer.as_ptr(),
                self.name.as_ptr(),
                bytes.as_ptr().cast(),
                size,
            )
        }
    }

    pub(crate) fn delete(&self) -> bool {
        unsafe {
            sys::SteamAPI_ISteamRemoteStorage_FileDelete(
                self.storage.pointer.as_ptr(),
                self.name.as_ptr(),
            )
        }
    }
}
