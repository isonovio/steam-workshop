use std::num::NonZeroU32;
use std::sync::{Mutex, MutexGuard};

use steamworks::Client;
use thiserror::Error;

use crate::Workshop;
use crate::features::RemoteStorage;

static SESSION: Mutex<()> = Mutex::new(());

pub struct Session {
    // Drop the client before releasing exclusive access to the Steam session.
    client: Client,
    _guard: MutexGuard<'static, ()>,
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Steam session is already active or unavailable")]
    Unavailable,
    #[error("Steam initialization failed: {0}")]
    Initialization(#[from] steamworks::SteamAPIInitError),
    #[error("Steam initialization failed: requested app {expected}, got {actual}")]
    AppMismatch { expected: u32, actual: u32 },
    #[error("Steam user is not logged in")]
    NotLoggedIn,
}

impl Session {
    /// The process must be launched with SteamAppId and SteamGameId set to app_id.
    pub(crate) fn connect(app_id: NonZeroU32) -> Result<Self, SessionError> {
        let _guard = SESSION.try_lock().map_err(|_| SessionError::Unavailable)?;
        let client = Client::init()?;
        let client_app_id = client.utils().app_id().0;
        if client_app_id != app_id.get() {
            return Err(SessionError::AppMismatch {
                expected: app_id.get(),
                actual: client_app_id,
            });
        }
        if !client.user().logged_on() {
            return Err(SessionError::NotLoggedIn);
        }
        Ok(Self { client, _guard })
    }

    pub fn workshop(&mut self) -> Workshop<'_> {
        Workshop::new(self)
    }

    pub fn remote_storage(&mut self) -> Option<RemoteStorage<'_>> {
        RemoteStorage::new(self)
    }

    pub(crate) fn pipe(&self) -> steamworks::sys::HSteamPipe {
        // The private Client keeps the initialized Steam session alive.
        unsafe { steamworks::sys::SteamAPI_GetHSteamPipe() }
    }

    pub(crate) fn app_id(&self) -> u32 {
        self.client.utils().app_id().0
    }

    pub fn user_id(&self) -> u64 {
        self.client.user().steam_id().raw()
    }
}

pub fn is_running() -> bool {
    // This status query requires no initialized session or pointer arguments.
    unsafe { steamworks::sys::SteamAPI_IsSteamRunning() }
}

#[cfg(test)]
#[path = "tests/session/tests.rs"]
mod tests;
