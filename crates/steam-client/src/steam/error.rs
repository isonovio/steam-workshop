use steamworks::SteamError;
use thiserror::Error;

use crate::native::ApiError;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CallError {
    #[error("Unable to allocate download buffer")]
    Allocation,
    #[error("Steam call did not start")]
    InvalidHandle,
    #[error("Steam call failed in transport")]
    Transport,
    #[error("Invalid Steam call result")]
    InvalidResult,
    #[error("Steam call failed: {0}")]
    Steam(SteamError),
    #[error("Steam call timed out; remote outcome is unknown")]
    Timeout,
}

impl From<ApiError> for CallError {
    fn from(error: ApiError) -> Self {
        match error {
            ApiError::Transport => Self::Transport,
            ApiError::InvalidResult => Self::InvalidResult,
            ApiError::Steam(error) => Self::Steam(error),
        }
    }
}
