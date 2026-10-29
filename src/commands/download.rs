use std::fs;
use std::io::{self, Write};
use std::num::{NonZeroU32, NonZeroU64};
use std::path::Path;
use std::time::Duration;

use steam_client::Session;
use tempfile::NamedTempFile;

use super::call_client;

pub fn command(app_id: NonZeroU32, item_id: NonZeroU64, output: &Path) -> i32 {
    call_client(app_id, |session| download(session, item_id, output))
}

fn download(session: &mut Session, item_id: NonZeroU64, output: &Path) -> Result<(), io::Error> {
    let timeout = Duration::from_secs(30);
    let download = || session.download(item_id, timeout).map_err(io::Error::other);
    let bytes = persist(output, download)?;

    println!("Downloaded {} bytes to {}", bytes.len(), output.display());
    Ok(())
}

fn persist(
    path: &Path,
    bytes: impl FnOnce() -> Result<Vec<u8>, io::Error>,
) -> Result<Vec<u8>, io::Error> {
    let mut temp_file = temp_sibling_file(path)?;

    let bytes = bytes()?;

    temp_file.write_all(&bytes)?;
    match temp_file.persist_noclobber(path) {
        Ok(_) => Ok(bytes),
        Err(err) => Err(err.error),
    }
}

fn temp_sibling_file(path: &Path) -> Result<NamedTempFile, io::Error> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Output file already exists",
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }?;
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    tempfile::NamedTempFile::new_in(parent)
}
