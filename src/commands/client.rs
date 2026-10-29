use std::{error::Error, num::NonZeroU32};

use steam_client::Session;

pub fn call_client<E: Error + 'static>(
    app_id: NonZeroU32,
    callback: impl FnOnce(&mut Session) -> Result<(), E>,
) -> i32 {
    match steam_client::run(app_id, callback) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            1
        }
    }
}
