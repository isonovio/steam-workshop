use thiserror::Error;

use crate::{CallError, CloudFile, CreateFileError, Executor, RemoteStorage};

use crate::utils::random_u128;

use super::Workshop;

pub enum CloudOperation<T> {
    Complete(Result<T, CallError>),
    Uncertain(CallError),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CloudError {
    #[error("Unable to generate temporary filename: {0}")]
    Random(String),
    #[error("Steam Cloud interface is unavailable")]
    Unavailable,
    #[error(transparent)]
    Create(#[from] CreateFileError),
    #[error(transparent)]
    Call(#[from] CallError),
    #[error("Failed to delete temporary Cloud file {filename}; operation error: {operation:?}")]
    Cleanup {
        filename: String,
        #[source]
        operation: Option<CallError>,
    },
    #[error("Remote outcome is unknown; temporary Cloud file retained: {filename}: {source}")]
    Uncertain {
        filename: String,
        #[source]
        source: CallError,
    },
}

struct TemporaryCloudFile<'session> {
    file: CloudFile<'session>,
}

impl Workshop<'_> {
    pub fn with_temporary_cloud_file<T>(
        &mut self,
        bytes: &[u8],
        operation: impl for<'runtime> FnOnce(
            RemoteStorage<'runtime>,
            &str,
            &mut Executor<'runtime>,
        ) -> CloudOperation<T>,
    ) -> Result<T, CloudError> {
        let (steam, mut executor) = self.session.access();
        let storage = steam.remote_storage().ok_or(CloudError::Unavailable)?;
        let temporary = TemporaryCloudFile::new(storage, bytes)?;
        let outcome = operation(storage, temporary.file.name(), &mut executor);
        temporary.finish(outcome)
    }
}

impl<'session> TemporaryCloudFile<'session> {
    fn new(storage: RemoteStorage<'session>, bytes: &[u8]) -> Result<Self, CloudError> {
        let name = temporary_filename()?;
        let file = CloudFile::new(storage, &name, bytes)?;
        Ok(Self { file })
    }

    fn finish<T>(self, outcome: CloudOperation<T>) -> Result<T, CloudError> {
        finish(outcome, self.file.name().to_owned(), || {
            self.file.delete().is_ok()
        })
    }
}

fn temporary_filename() -> Result<String, CloudError> {
    let random = random_u128().map_err(|error| CloudError::Random(error.to_string()))?;
    Ok(format!("steam-workshop-{random:032x}.tmp"))
}

fn finish<T>(
    outcome: CloudOperation<T>,
    filename: String,
    cleanup: impl FnOnce() -> bool,
) -> Result<T, CloudError> {
    let result = match outcome {
        CloudOperation::Complete(result) => result,
        CloudOperation::Uncertain(source) => {
            return Err(CloudError::Uncertain { filename, source });
        }
    };
    if !cleanup() {
        return Err(CloudError::Cleanup {
            filename,
            operation: result.err(),
        });
    }
    result.map_err(CloudError::Call)
}

#[cfg(test)]
#[path = "../tests/workshop/staging/tests.rs"]
mod tests;
