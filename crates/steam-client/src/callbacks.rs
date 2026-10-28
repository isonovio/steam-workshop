use std::mem::{MaybeUninit, size_of};
use std::thread;
use std::time::{Duration, Instant};

use steamworks::{SteamError, sys};
use thiserror::Error;

use crate::Session;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CallError {
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

pub struct PendingCall<'a, T> {
    pub(crate) session: &'a mut Session,
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
        let pipe = self.session.pipe();
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
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires Steam running with SteamAppId and SteamGameId set to 286160"]
    fn live_legacy_call() {
        let mut session = Session::connect(std::num::NonZeroU32::new(286160).unwrap()).unwrap();
        // The session owns the interface; this request only reads item metadata.
        let handle = unsafe {
            let storage = sys::SteamAPI_SteamRemoteStorage_v016();
            assert!(!storage.is_null());
            sys::SteamAPI_ISteamRemoteStorage_GetPublishedFileDetails(storage, 3826709471, 0)
        };
        let call = PendingCall {
            session: &mut session,
            handle,
            callback_id: sys::RemoteStorageGetPublishedFileDetailsResult_t_k_iCallback as i32,
            result_size: size_of::<sys::RemoteStorageGetPublishedFileDetailsResult_t>() as i32,
            decode: |bytes| {
                // The waiter checked the callback ID and exact native result size.
                let result = unsafe {
                    bytes
                        .as_ptr()
                        .cast::<sys::RemoteStorageGetPublishedFileDetailsResult_t>()
                        .read_unaligned()
                };
                match result.m_eResult {
                    sys::EResult::k_EResultOK => Ok(result.m_nPublishedFileId),
                    error => Err(CallError::Steam(error.into())),
                }
            },
        };
        assert_eq!(call.wait(Duration::from_secs(30)), Ok(3826709471));
    }

    #[test]
    fn completes_after_pending_poll() {
        let mut polls = 0;
        let result = wait_until(Duration::from_secs(1), || {
            polls += 1;
            Ok((polls == 2).then_some(42))
        });
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn timeout_does_not_poll_or_retry() {
        let result = wait_until::<()>(Duration::ZERO, || panic!("polled after deadline"));
        assert_eq!(result, Err(CallError::Timeout));
        assert_eq!(
            wait_until::<()>(Duration::from_millis(1), || Ok(None)),
            Err(CallError::Timeout)
        );
    }

    #[test]
    fn failed_results_are_not_decoded() {
        let decode = |_: &[u8]| -> Result<(), CallError> { panic!("decoded failed result") };
        assert_eq!(
            decode_result(true, true, &[], decode),
            Err(CallError::Transport)
        );
        assert_eq!(
            decode_result(false, false, &[], decode),
            Err(CallError::InvalidResult)
        );
    }

    #[test]
    fn steam_errors_are_preserved() {
        let error = decode_result(true, false, &[], |_| -> Result<(), CallError> {
            Err(CallError::Steam(SteamError::AccessDenied))
        });
        assert_eq!(error, Err(CallError::Steam(SteamError::AccessDenied)));
        assert_eq!(
            wait_until::<()>(Duration::from_secs(1), || Err(CallError::Transport)),
            Err(CallError::Transport)
        );
    }
}
