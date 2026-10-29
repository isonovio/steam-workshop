use std::ffi::CString;
use std::ptr::NonNull;

use steamworks::sys;
use thiserror::Error;

use crate::Session;

use super::CloudFile;

pub struct RemoteStorage<'session> {
    pub(super) pointer: NonNull<sys::ISteamRemoteStorage>,
    pub(super) session: &'session mut Session,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CreateFileError {
    #[error("Invalid Cloud filename")]
    InvalidName,
    #[error("Cloud file already exists")]
    AlreadyExists,
    #[error("Input exceeds the upload limit")]
    TooLarge,
    #[error("Failed to write Cloud file")]
    Write,
    #[error("Write failed and partial file cleanup failed: {filename}")]
    Cleanup { filename: String },
}

impl<'session> RemoteStorage<'session> {
    pub(crate) fn new(session: &'session mut Session) -> Option<Self> {
        // The exclusive session borrow keeps the SDK alive for this interface.
        let pointer = unsafe { sys::SteamAPI_SteamRemoteStorage_v016() };
        Some(Self {
            pointer: NonNull::new(pointer)?,
            session,
        })
    }

    /// Creates a new owned file. Steam has no atomic create-if-absent operation.
    pub fn write_new(
        self,
        name: &str,
        bytes: &[u8],
    ) -> Result<CloudFile<'session>, CreateFileError> {
        let name = filename(name).ok_or(CreateFileError::InvalidName)?;
        if bytes.len() > sys::k_unMaxCloudFileChunkSize as usize {
            return Err(CreateFileError::TooLarge);
        }
        if self.exists(&name) {
            return Err(CreateFileError::AlreadyExists);
        }
        // The validated filename and slice remain alive through the synchronous call.
        let written = unsafe {
            sys::SteamAPI_ISteamRemoteStorage_FileWrite(
                self.pointer.as_ptr(),
                name.as_ptr(),
                bytes.as_ptr().cast(),
                bytes.len() as i32,
            )
        };
        self.finish_write(name, written)
    }

    fn exists(&self, name: &CString) -> bool {
        // The interface retains the session and the name is NUL-terminated.
        unsafe {
            sys::SteamAPI_ISteamRemoteStorage_FileExists(self.pointer.as_ptr(), name.as_ptr())
        }
    }

    fn finish_write(
        self,
        name: CString,
        written: bool,
    ) -> Result<CloudFile<'session>, CreateFileError> {
        if written {
            return Ok(CloudFile::new(self, name));
        }
        if self.exists(&name) {
            let file = CloudFile::new(self, name);
            if let Err(file) = file.delete() {
                return Err(CreateFileError::Cleanup {
                    filename: file.retain(),
                });
            }
        }
        Err(CreateFileError::Write)
    }
}

fn filename(name: &str) -> Option<CString> {
    if name.is_empty() {
        return None;
    }
    CString::new(name).ok()
}

#[cfg(test)]
#[path = "../tests/features/remote_storage/tests.rs"]
mod tests;
