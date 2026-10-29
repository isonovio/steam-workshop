use std::ffi::CString;

use steamworks::{SteamError, sys};
use thiserror::Error;

use crate::{CallError, MAX_UPLOAD_SIZE, Session};

use crate::utils::random_u128;

#[derive(Debug, Error)]
pub enum CloudError {
    #[error("Input exceeds the {MAX_UPLOAD_SIZE}-byte upload limit")]
    TooLarge,
    #[error("Unable to generate temporary filename: {0}")]
    Random(String),
    #[error("Steam Cloud interface is unavailable")]
    Unavailable,
    #[error("Temporary Cloud filename already exists")]
    NameCollision,
    #[error(transparent)]
    Call(#[from] CallError),
    #[error("Failed to delete temporary Cloud file {filename}; operation error: {operation:?}")]
    Cleanup {
        filename: String,
        #[source]
        operation: Option<CallError>,
    },
    #[error("Steam call timed out; temporary Cloud file retained: {filename}")]
    Uncertain { filename: String },
}

impl Session {
    /// Stages bytes for one operation, then deletes the staging file on completion.
    /// A timeout retains the file because the remote operation may still need it.
    pub fn with_temporary_cloud_file<T>(
        &mut self,
        bytes: &[u8],
        operation: impl FnOnce(&mut Self, &str) -> Result<T, CallError>,
    ) -> Result<T, CloudError> {
        if bytes.len() > MAX_UPLOAD_SIZE {
            return Err(CloudError::TooLarge);
        }
        let filename = temporary_filename()?;
        let name = CString::new(filename.as_str()).unwrap();
        // The exclusive session keeps this interface alive through operation and cleanup.
        let storage = unsafe { sys::SteamAPI_SteamRemoteStorage_v016() };
        if storage.is_null() {
            return Err(CloudError::Unavailable);
        }
        let exists =
            unsafe { sys::SteamAPI_ISteamRemoteStorage_FileExists(storage, name.as_ptr()) };
        if exists {
            return Err(CloudError::NameCollision);
        }
        // The generated name is NUL-terminated and the checked byte count fits i32.
        let written = unsafe {
            sys::SteamAPI_ISteamRemoteStorage_FileWrite(
                storage,
                name.as_ptr(),
                bytes.as_ptr().cast(),
                bytes.len() as i32,
            )
        };
        let result = match written {
            true => operation(self, &filename),
            false => Err(CallError::Steam(SteamError::IOFailure)),
        };
        finish(result, filename, || {
            // FileWrite can fail before creating a file; absence already satisfies cleanup.
            unsafe {
                !sys::SteamAPI_ISteamRemoteStorage_FileExists(storage, name.as_ptr())
                    || sys::SteamAPI_ISteamRemoteStorage_FileDelete(storage, name.as_ptr())
            }
        })
    }
}

fn temporary_filename() -> Result<String, CloudError> {
    let random = random_u128().map_err(|error| CloudError::Random(error.to_string()))?;
    Ok(format!("steam-workshop-{random:032x}.tmp"))
}

fn finish<T>(
    result: Result<T, CallError>,
    filename: String,
    cleanup: impl FnOnce() -> bool,
) -> Result<T, CloudError> {
    if matches!(result, Err(CallError::Timeout)) {
        return Err(CloudError::Uncertain { filename });
    }
    if !cleanup() {
        return Err(CloudError::Cleanup {
            filename,
            operation: result.err(),
        });
    }
    result.map_err(CloudError::Call)
}

#[cfg(test)]
#[path = "tests/staging/tests.rs"]
mod tests;
