use std::fs::File;
use std::io::{self, Read};
use std::num::{NonZeroU32, NonZeroU64};
use std::path::Path;
use std::time::Duration;

use steam_client::{MAX_UPLOAD_SIZE, Session, Workshop};

use super::call_client;

pub fn command(app_id: NonZeroU32, item_id: NonZeroU64, file: &Path) -> i32 {
    call_client(app_id, |session| update_file(session, item_id, file))
}

fn update_file(session: &mut Session, item_id: NonZeroU64, path: &Path) -> Result<(), io::Error> {
    let bytes = read_input(path)?;
    let result = Workshop::new(session)
        .update_file(item_id, &bytes, Duration::from_secs(300))
        .map_err(io::Error::other)?;
    match result.needs_legal_agreement {
        true => println!(
            "Uploaded {} bytes to item {}; accept the Workshop legal agreement at https://steamcommunity.com/sharedfiles/filedetails/?id={}",
            bytes.len(),
            result.item_id,
            result.item_id
        ),
        false => println!("Updated item {} with {} bytes", result.item_id, bytes.len()),
    }
    Ok(())
}

fn read_input(path: &Path) -> Result<Vec<u8>, io::Error> {
    let metadata = path.metadata()?;
    if !metadata.is_file() {
        return invalid_input("Input must be a regular file");
    }
    if metadata.len() > MAX_UPLOAD_SIZE as u64 {
        return invalid_input("Input exceeds the upload limit");
    }

    let file = File::open(path)?;
    let mut bytes = Vec::new();
    file.take(MAX_UPLOAD_SIZE as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_UPLOAD_SIZE {
        return invalid_input("Input exceeds the upload limit");
    }

    Ok(bytes)
}

fn invalid_input<T>(why: &'static str) -> Result<T, io::Error> {
    Err(io::Error::new(io::ErrorKind::InvalidInput, why))
}

#[cfg(test)]
#[path = "../tests/commands/update_file/tests.rs"]
mod tests;
