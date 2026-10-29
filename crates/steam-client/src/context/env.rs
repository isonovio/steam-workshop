use std::env;
use std::ffi::OsString;

pub(super) const CHILD_APP_ID: &str = "STEAM_WORKSHOP_CHILD_APP_ID";

pub(super) struct Environment {
    pub child_app_id: Option<OsString>,
    pub steam_app_id: Option<OsString>,
    pub steam_game_id: Option<OsString>,
}

impl Environment {
    pub fn read() -> Self {
        Self {
            child_app_id: env::var_os(CHILD_APP_ID),
            steam_app_id: env::var_os("SteamAppId"),
            steam_game_id: env::var_os("SteamGameId"),
        }
    }
}
