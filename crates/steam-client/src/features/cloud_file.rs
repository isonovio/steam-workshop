use std::ffi::CString;

use steamworks::sys;

use super::RemoteStorage;

pub struct CloudFile<'session> {
    storage: RemoteStorage<'session>,
    name: CString,
}

impl<'session> CloudFile<'session> {
    pub(super) fn new(storage: RemoteStorage<'session>, name: CString) -> Self {
        Self { storage, name }
    }

    pub fn name(&self) -> &str {
        self.name.to_str().unwrap()
    }

    pub(crate) fn with_session<T>(
        &mut self,
        operation: impl FnOnce(&mut crate::Session, &str) -> T,
    ) -> T {
        operation(self.storage.session, self.name.to_str().unwrap())
    }

    pub fn read(&self) -> Option<Vec<u8>> {
        // The interface and filename remain valid while this owned file is borrowed.
        let exists = unsafe {
            sys::SteamAPI_ISteamRemoteStorage_FileExists(
                self.storage.pointer.as_ptr(),
                self.name.as_ptr(),
            )
        };
        if !exists {
            return None;
        }
        let size = unsafe {
            sys::SteamAPI_ISteamRemoteStorage_GetFileSize(
                self.storage.pointer.as_ptr(),
                self.name.as_ptr(),
            )
        };
        let size: usize = size.try_into().ok()?;
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(size).ok()?;
        bytes.resize(size, 0);
        // The validated size fits i32 and matches the writable buffer.
        let read = unsafe {
            sys::SteamAPI_ISteamRemoteStorage_FileRead(
                self.storage.pointer.as_ptr(),
                self.name.as_ptr(),
                bytes.as_mut_ptr().cast(),
                size as i32,
            )
        };
        (read == size as i32).then_some(bytes)
    }

    /// Consumes the file on success; returns ownership if Steam refuses deletion.
    pub fn delete(self) -> Result<(), Self> {
        // The session and filename remain live until after the deletion attempt.
        let deleted = unsafe {
            sys::SteamAPI_ISteamRemoteStorage_FileDelete(
                self.storage.pointer.as_ptr(),
                self.name.as_ptr(),
            )
        };
        match deleted {
            true => Ok(()),
            false => Err(self),
        }
    }

    /// Relinquishes the local owner without deleting the remote file.
    pub fn retain(self) -> String {
        self.name.into_string().unwrap()
    }
}
