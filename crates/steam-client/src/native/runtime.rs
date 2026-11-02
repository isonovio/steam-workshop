use std::marker::PhantomData;
use std::rc::Rc;

use steamworks::{Client, sys};

pub(crate) use steamworks::SteamAPIInitError as InitializationError;

pub(crate) struct Runtime {
    client: Client,
    // Native dispatch and interface access stay on the initializing thread.
    _thread: PhantomData<Rc<()>>,
}

impl Runtime {
    pub(crate) fn new() -> Result<Self, InitializationError> {
        Ok(Self {
            client: Client::init()?,
            _thread: PhantomData,
        })
    }

    pub(crate) fn app_id(&self) -> u32 {
        self.client.utils().app_id().0
    }

    pub(crate) fn logged_on(&self) -> bool {
        self.client.user().logged_on()
    }

    pub(crate) fn user_id(&self) -> u64 {
        self.client.user().steam_id().raw()
    }

    pub(crate) fn pipe(&self) -> sys::HSteamPipe {
        // The owned client keeps the SDK initialized.
        unsafe { sys::SteamAPI_GetHSteamPipe() }
    }
}

pub fn is_running() -> bool {
    unsafe { sys::SteamAPI_IsSteamRunning() }
}
