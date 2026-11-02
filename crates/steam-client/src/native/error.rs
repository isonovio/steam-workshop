use steamworks::SteamError;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum ApiError {
    #[error("Steam call failed in transport")]
    Transport,
    #[error("Invalid Steam call result")]
    InvalidResult,
    #[error("Steam call failed: {0}")]
    Steam(SteamError),
}
