use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::io;
use std::num::NonZeroU32;
use std::process::{Command, ExitStatus, Stdio};

use thiserror::Error;

use crate::env::{CHILD_APP_ID, Environment};
use crate::{Session, SessionError};

#[derive(Debug, Error)]
pub enum RunError<E: Error + 'static> {
    #[error("Invalid Steam child process configuration")]
    ChildConfiguration,
    #[error("Failed to launch or wait for Steam command: {0}")]
    Process(#[source] io::Error),
    #[error("Steam command terminated: {0}")]
    Terminated(ExitStatus),
    #[error(transparent)]
    Initialization(#[from] SessionError),
    #[error(transparent)]
    Command(E),
}

/// Re-executes the current command with Steam configured, then runs its body once.
/// Call this after argument parsing and before command side effects. The body runs
/// in the child; only its output and exit code return to the parent. Return from
/// the body normally so its session is shut down before the executable exits.
pub fn run<E: Error + 'static>(
    app_id: NonZeroU32,
    command: impl FnOnce(&mut Session) -> Result<(), E>,
) -> Result<i32, RunError<E>> {
    dispatch(app_id, || {
        let mut session = Session::connect(app_id)?;
        command(&mut session).map_err(RunError::Command)
    })
}

fn dispatch<E: Error + 'static>(
    app_id: NonZeroU32,
    command: impl FnOnce() -> Result<(), RunError<E>>,
) -> Result<i32, RunError<E>> {
    let environment = Environment::read();
    let app_id = app_id.to_string();
    match environment.child_app_id {
        Some(marker) => {
            let expected = Some(OsStr::new(&app_id));
            if marker != app_id.as_str()
                || environment.steam_app_id.as_deref() != expected
                || environment.steam_game_id.as_deref() != expected
            {
                return Err(RunError::ChildConfiguration);
            }
            command()?;
            Ok(0)
        }
        None => {
            let status = Command::new(env::current_exe().map_err(RunError::Process)?)
                .args(env::args_os().skip(1))
                .env(CHILD_APP_ID, &app_id)
                .env("SteamAppId", &app_id)
                .env("SteamGameId", &app_id)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .map_err(RunError::Process)?;
            status.code().ok_or(RunError::Terminated(status))
        }
    }
}

#[cfg(test)]
#[path = "tests/run/tests.rs"]
mod tests;
