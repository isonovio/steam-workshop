use std::thread;
use std::time::{Duration, Instant};

use steamworks::SteamError;
use thiserror::Error;

use crate::native::{ApiCall, ApiError, Runtime};

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

pub(crate) struct PendingCall<'a, T> {
    pub(crate) runtime: &'a mut Runtime,
    pub(crate) call: ApiCall<'a, T>,
}

impl<T> PendingCall<'_, T> {
    /// A timeout stops waiting; it does not cancel the remote operation.
    pub fn wait(self, timeout: Duration) -> Result<T, CallError> {
        wait_until(timeout, || {
            self.call.poll(self.runtime).map_err(CallError::from)
        })
    }
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

fn wait_until<T>(
    timeout: Duration,
    mut poll: impl FnMut() -> Result<Option<T>, CallError>,
) -> Result<T, CallError> {
    let start = Instant::now();
    loop {
        if start.elapsed() >= timeout {
            return Err(CallError::Timeout);
        }
        if let Some(result) = poll()? {
            return Ok(result);
        }
        thread::sleep(Duration::from_millis(10).min(timeout.saturating_sub(start.elapsed())));
    }
}

#[cfg(test)]
#[path = "../tests/features/callbacks/tests.rs"]
mod tests;
