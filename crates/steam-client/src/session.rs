use std::num::NonZeroU32;
use std::sync::{Mutex, MutexGuard};

use steamworks::Client;
use thiserror::Error;

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
    #[error("Steam initialized for app {actual}, expected {expected}")]
    AppMismatch { expected: u32, actual: u32 },
    #[error("Steam user is not logged in")]
    NotLoggedIn,
}

impl Session {
    /// The process must be launched with SteamAppId and SteamGameId set to app_id.
    pub fn connect(app_id: NonZeroU32) -> Result<Self, SessionError> {
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

    pub fn user_id(&self) -> u64 {
        self.client.user().steam_id().raw()
    }
}

pub fn is_running() -> bool {
    // This status query requires no initialized session or pointer arguments.
    unsafe { steamworks::sys::SteamAPI_IsSteamRunning() }
}
