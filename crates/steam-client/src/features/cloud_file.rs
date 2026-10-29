use std::ffi::CString;

use steamworks::sys;
use thiserror::Error;

use super::RemoteStorage;

pub struct CloudFile<'session> {
    storage: RemoteStorage<'session>,
    name: String,
    name_cstr: CString,
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

impl<'session> CloudFile<'session> {
    /// Creates a new owned file. Steam has no atomic create-if-absent operation.
    pub fn new(
        storage: RemoteStorage<'session>,
        name: &str,
        bytes: &[u8],
    ) -> Result<Self, CreateFileError> {
        let name_cstr = to_filename(name).ok_or(CreateFileError::InvalidName)?;

        if bytes.len() > sys::k_unMaxCloudFileChunkSize as usize {
            return Err(CreateFileError::TooLarge);
        }
        if storage.exists(&name_cstr) {
            return Err(CreateFileError::AlreadyExists);
        }
        if !storage.write(&name_cstr, bytes) {
            if storage.exists(&name_cstr) && !storage.delete(&name_cstr) {
                return Err(CreateFileError::Cleanup {
                    filename: name.to_owned(),
                });
            }
            return Err(CreateFileError::Write);
        }

        Ok(Self {
            storage,
            name: name.to_owned(),
            name_cstr,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn with_session<T>(
        &mut self,
        operation: impl FnOnce(&mut crate::Session, &str) -> T,
    ) -> T {
        operation(self.storage.session, &self.name)
    }

    pub fn read(&self) -> Option<Vec<u8>> {
        self.storage.read(&self.name_cstr)
    }

    /// Consumes the file on success; returns ownership if Steam refuses deletion.
    pub fn delete(self) -> Result<(), Self> {
        // The owned storage keeps the session alive during deletion.
        match self.storage.delete(&self.name_cstr) {
            true => Ok(()),
            false => Err(self),
        }
    }

    /// Relinquishes the local owner without deleting the remote file.
    pub fn retain(self) -> String {
        self.name
    }
}

fn to_filename(name: &str) -> Option<CString> {
    match name.is_empty() {
        true => None,
        false => CString::new(name).ok(),
    }
}

#[cfg(test)]
#[path = "../tests/features/cloud_file/tests.rs"]
mod tests;
