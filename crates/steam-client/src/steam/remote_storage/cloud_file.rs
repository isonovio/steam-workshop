use std::ffi::CString;
use std::marker::PhantomData;

use thiserror::Error;

use crate::native::remote_storage::File;
use crate::utils::try_reserve_exact;

use super::{MAX_UPLOAD_SIZE, RemoteStorage};

pub struct CloudFile<'session, 'storage> {
    pub(super) native: File<'session>,
    name: String,
    _brand: PhantomData<fn(&'storage ()) -> &'storage ()>,
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

impl<'session, 'storage> CloudFile<'session, 'storage> {
    /// Creates a new owned file. Steam has no atomic create-if-absent operation.
    pub fn new(
        storage: RemoteStorage<'session, 'storage>,
        name: &str,
        bytes: &[u8],
    ) -> Result<Self, CreateFileError> {
        let name_cstr = to_filename(name).ok_or(CreateFileError::InvalidName)?;

        if bytes.len() > MAX_UPLOAD_SIZE {
            return Err(CreateFileError::TooLarge);
        }
        let file = storage.native.file(name_cstr);
        if file.exists() {
            return Err(CreateFileError::AlreadyExists);
        }
        if !file.write(bytes) {
            if file.exists() && !file.delete() {
                return Err(CreateFileError::Cleanup {
                    filename: name.to_owned(),
                });
            }
            return Err(CreateFileError::Write);
        }

        Ok(Self {
            native: file,
            name: name.to_owned(),
            _brand: storage.brand,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn read(&self) -> Option<Vec<u8>> {
        let file = &self.native;
        file.exists().then_some(())?;
        let size: usize = file.get_size().try_into().ok()?;
        let mut buffer = try_reserve_exact(size)?;
        let read = file.read(&mut buffer)?;
        (read == size as i32).then_some(buffer)
    }

    /// Consumes the file on success; returns ownership if Steam refuses deletion.
    pub fn delete(self) -> Result<(), Self> {
        match self.native.delete() {
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
#[path = "../../tests/steam/remote_storage/cloud_file/tests.rs"]
mod tests;
