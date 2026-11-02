use std::mem::{MaybeUninit, size_of};
use std::thread;
use std::time::{Duration, Instant};

use steamworks::{SteamError, sys};
use thiserror::Error;

use crate::native::Runtime;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CallError {
    #[error("Unable to allocate download buffer")]
    Allocation,
    #[error("Steam call did not start")]
    InvalidHandle,
    #[error("Steam call failed in transport")]
    Transport,
    #[error("Invalid Steam call result")]
    InvalidResult,
    #[error("Steam call failed: {0}")]
    Steam(SteamError),
    #[error("Steam call timed out; remote outcome is unknown")]
    Timeout,
}

pub(crate) struct PendingCall<'a, T> {
    pub(crate) runtime: &'a mut Runtime,
    pub(crate) handle: sys::SteamAPICall_t,
    pub(crate) callback_id: i32,
    pub(crate) result_size: i32,
    pub(crate) decode: fn(&[u8]) -> Result<T, CallError>,
}

impl<T> PendingCall<'_, T> {
    /// A timeout stops waiting; it does not cancel the remote operation.
    pub fn wait(self, timeout: Duration) -> Result<T, CallError> {
        if self.handle == 0 {
            return Err(CallError::InvalidHandle);
        }
        wait_until(timeout, || self.poll())
    }

    fn poll(&self) -> Result<Option<T>, CallError> {
        let pipe = self.runtime.pipe();
        let mut message = MaybeUninit::<sys::CallbackMsg_t>::uninit();
        // The borrowed session owns this pipe and initialized manual dispatch.
        unsafe {
            sys::SteamAPI_ManualDispatch_RunFrame(pipe);
            if !sys::SteamAPI_ManualDispatch_GetNextCallback(pipe, message.as_mut_ptr()) {
                return Ok(None);
            }
            let _release = CallbackRelease(pipe);
            let message = message.assume_init();
            if message.m_iCallback != sys::SteamAPICallCompleted_t_k_iCallback as i32 {
                return Ok(None);
            }
            if message.m_pubParam.is_null()
                || message.m_cubParam != size_of::<sys::SteamAPICallCompleted_t>() as i32
            {
                return Err(CallError::InvalidResult);
            }
            // Callback storage can be unaligned and is valid until _release drops.
            let completed = message
                .m_pubParam
                .cast::<sys::SteamAPICallCompleted_t>()
                .read_unaligned();
            if completed.m_hAsyncCall != self.handle {
                return Ok(None);
            }
            if completed.m_iCallback != self.callback_id
                || self.result_size <= 0
                || completed.m_cubParam != self.result_size as u32
            {
                return Err(CallError::InvalidResult);
            }
            let mut bytes = vec![0; self.result_size as usize];
            let mut failed = false;
            let retrieved = sys::SteamAPI_ManualDispatch_GetAPICallResult(
                pipe,
                self.handle,
                bytes.as_mut_ptr().cast(),
                self.result_size,
                self.callback_id,
                &mut failed,
            );
            decode_result(retrieved, failed, &bytes, self.decode).map(Some)
        }
    }
}

struct CallbackRelease(sys::HSteamPipe);

impl Drop for CallbackRelease {
    fn drop(&mut self) {
        // Created only after GetNextCallback succeeds, while the session is alive.
        unsafe { sys::SteamAPI_ManualDispatch_FreeLastCallback(self.0) }
    }
}

fn decode_result<T>(
    retrieved: bool,
    failed: bool,
    bytes: &[u8],
    decode: fn(&[u8]) -> Result<T, CallError>,
) -> Result<T, CallError> {
    if failed {
        return Err(CallError::Transport);
    }
    if !retrieved {
        return Err(CallError::InvalidResult);
    }
    decode(bytes)
}

fn wait_until<T>(
    timeout: Duration,
    mut poll: impl FnMut() -> Result<Option<T>, CallError>,
) -> Result<T, CallError> {
    let start = Instant::now();
    loop {
        if start.elapsed() >= timeout {
            return Err(CallError::Timeout);
        }
        if let Some(result) = poll()? {
            return Ok(result);
        }
        thread::sleep(Duration::from_millis(10).min(timeout.saturating_sub(start.elapsed())));
    }
}

#[cfg(test)]
#[path = "../tests/features/callbacks/tests.rs"]
mod tests;
