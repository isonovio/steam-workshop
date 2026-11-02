use std::num::NonZeroU32;
use std::sync::{Mutex, MutexGuard};

use thiserror::Error;

use crate::Workshop;
use crate::native::{InitializationError, Runtime};

use super::{Access, Executor};

static SESSION: Mutex<()> = Mutex::new(());

pub struct Session {
    // Drop the runtime before releasing exclusive access to the Steam session.
    runtime: Runtime,
    dispatch_access: (),
    _guard: MutexGuard<'static, ()>,
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Steam session is already active or unavailable")]
    Unavailable,
    #[error("Steam initialization failed: {0}")]
    Initialization(#[from] InitializationError),
    #[error("Steam initialization failed: requested app {expected}, got {actual}")]
    AppMismatch { expected: u32, actual: u32 },
    #[error("Steam user is not logged in")]
    NotLoggedIn,
}

impl Session {
    /// The process must be launched with SteamAppId and SteamGameId set to app_id.
    pub(crate) fn connect(app_id: NonZeroU32) -> Result<Self, SessionError> {
        let _guard = SESSION.try_lock().map_err(|_| SessionError::Unavailable)?;
        let runtime = Runtime::new()?;
        let client_app_id = runtime.app_id();
        if client_app_id != app_id.get() {
            return Err(SessionError::AppMismatch {
                expected: app_id.get(),
                actual: client_app_id,
            });
        }
        if !runtime.logged_on() {
            return Err(SessionError::NotLoggedIn);
        }
        Ok(Self {
            runtime,
            dispatch_access: (),
            _guard,
        })
    }

    pub fn workshop(&mut self) -> Workshop<'_> {
        Workshop::new(self)
    }

    pub fn access(&mut self) -> (Access<'_>, Executor<'_>) {
        let access = Access::new(&self.runtime);
        // The session owns the sole dispatch token; the returned borrow prevents reuse.
        let executor = unsafe { Executor::new(&self.runtime, &mut self.dispatch_access) };
        (access, executor)
    }

    pub(crate) fn app_id(&self) -> u32 {
        self.runtime.app_id()
    }

    pub fn user_id(&self) -> u64 {
        self.runtime.user_id()
    }
}

#[cfg(test)]
#[path = "../tests/steam/session/tests.rs"]
mod tests;
