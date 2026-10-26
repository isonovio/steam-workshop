pub fn is_running() -> bool {
    // This status query requires no initialized session or pointer arguments.
    unsafe { steamworks_sys::SteamAPI_IsSteamRunning() }
}
