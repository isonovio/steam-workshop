use std::thread;
use std::time::Duration;

use crate::native::{ApiCall, ApiError, Dispatch, Runtime};
use crate::utils::Timeout;

use super::CallError;

#[doc = include_str!("../tests/steam/executor/contracts.md")]
pub struct Executor<'runtime> {
    dispatch: Dispatch<'runtime>,
    _access: &'runtime mut (),
}

impl<'runtime> Executor<'runtime> {
    /// The access token must be unique for the initialized runtime.
    pub(crate) unsafe fn new(runtime: &'runtime Runtime, access: &'runtime mut ()) -> Self {
        Self {
            dispatch: unsafe { Dispatch::new(runtime) },
            _access: access,
        }
    }

    pub(super) fn wait<T>(
        &mut self,
        call: &ApiCall<'_, T>,
        timeout: Timeout,
    ) -> Result<T, CallError> {
        wait_until(timeout, || self.poll(call).map_err(CallError::from))
    }

    fn poll<T>(&mut self, call: &ApiCall<'_, T>) -> Result<Option<T>, ApiError> {
        self.dispatch.run_frame();
        let Some(callback) = self.dispatch.next() else {
            return Ok(None);
        };
        call.resolve(&callback)
    }
}

fn wait_until<T>(
    timeout: Timeout,
    mut poll: impl FnMut() -> Result<Option<T>, CallError>,
) -> Result<T, CallError> {
    loop {
        timeout.check().ok_or(CallError::Timeout)?;
        if let Some(result) = poll()? {
            return Ok(result);
        }
        thread::sleep(Duration::from_millis(10).min(timeout.remaining()));
    }
}

#[cfg(test)]
#[path = "../tests/steam/executor/tests.rs"]
mod tests;
