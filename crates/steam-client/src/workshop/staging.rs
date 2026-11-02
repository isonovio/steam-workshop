use thiserror::Error;

use crate::steam::UpdateOutcome;
use crate::utils::random_u128;
use crate::{CallError, CloudFile, CreateFileError};

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

pub(super) fn temporary_filename() -> Result<String, CloudError> {
    let random = random_u128().map_err(|error| CloudError::Random(error.to_string()))?;
    Ok(format!("steam-workshop-{random:032x}.tmp"))
}

pub(super) fn finish(outcome: UpdateOutcome<'_, '_>) -> Result<bool, CloudError> {
    match outcome {
        UpdateOutcome::NotSubmitted { file, error } => cleanup(file, Err(error)),
        UpdateOutcome::Complete { file, result } => cleanup(file, result),
        UpdateOutcome::Uncertain { filename, error } => Err(CloudError::Uncertain {
            filename,
            source: error,
        }),
    }
}

pub(super) fn cleanup(
    file: CloudFile<'_, '_>,
    result: Result<bool, CallError>,
) -> Result<bool, CloudError> {
    match file.delete() {
        Ok(()) => result.map_err(CloudError::Call),
        Err(file) => Err(CloudError::Cleanup {
            filename: file.retain(),
            operation: result.err(),
        }),
    }
}

#[cfg(test)]
#[path = "../tests/workshop/staging/tests.rs"]
mod tests;
