use std::marker::PhantomData;
use std::mem::{MaybeUninit, size_of};

use steamworks::sys;

use super::{ApiError, Runtime};

pub(super) struct Dispatch<'runtime> {
    pipe: sys::HSteamPipe,
    _runtime: PhantomData<&'runtime mut Runtime>,
}

pub(super) struct Callback<'dispatch> {
    pipe: sys::HSteamPipe,
    message: sys::CallbackMsg_t,
    _dispatch: PhantomData<&'dispatch mut ()>,
}

pub(super) struct Completion {
    pub handle: u64,
    pub callback_id: i32,
    pub size: u32,
}

impl<'runtime> Dispatch<'runtime> {
    pub(super) fn new(runtime: &'runtime mut Runtime) -> Self {
        Self {
            pipe: runtime.pipe(),
            _runtime: PhantomData,
        }
    }

    pub(super) fn run_frame(&mut self) {
        unsafe { sys::SteamAPI_ManualDispatch_RunFrame(self.pipe) }
    }

    pub(super) fn next(&mut self) -> Option<Callback<'_>> {
        let mut message = MaybeUninit::uninit();
        let retrieved = unsafe {
            sys::SteamAPI_ManualDispatch_GetNextCallback(self.pipe, message.as_mut_ptr())
        };
        retrieved.then(|| Callback {
            pipe: self.pipe,
            message: unsafe { message.assume_init() },
            _dispatch: PhantomData,
        })
    }
}

impl Callback<'_> {
    pub(super) fn completion(&self) -> Result<Option<Completion>, ApiError> {
        if self.message.m_iCallback != sys::SteamAPICallCompleted_t_k_iCallback as i32 {
            return Ok(None);
        }
        if self.message.m_pubParam.is_null()
            || self.message.m_cubParam != size_of::<sys::SteamAPICallCompleted_t>() as i32
        {
            return Err(ApiError::InvalidResult);
        }
        // The lease keeps the possibly unaligned native buffer alive.
        let completed = unsafe {
            self.message
                .m_pubParam
                .cast::<sys::SteamAPICallCompleted_t>()
                .read_unaligned()
        };
        Ok(Some(Completion {
            handle: completed.m_hAsyncCall,
            callback_id: completed.m_iCallback,
            size: completed.m_cubParam,
        }))
    }

    pub(super) fn result(
        &self,
        handle: u64,
        callback_id: i32,
        buffer: &mut [u8],
    ) -> Result<(), ApiError> {
        let size = buffer
            .len()
            .try_into()
            .map_err(|_| ApiError::InvalidResult)?;
        let mut failed = false;
        let retrieved = unsafe {
            sys::SteamAPI_ManualDispatch_GetAPICallResult(
                self.pipe,
                handle,
                buffer.as_mut_ptr().cast(),
                size,
                callback_id,
                &mut failed,
            )
        };
        retrieval_status(retrieved, failed)
    }
}

impl Drop for Callback<'_> {
    fn drop(&mut self) {
        unsafe { sys::SteamAPI_ManualDispatch_FreeLastCallback(self.pipe) }
    }
}

fn retrieval_status(retrieved: bool, failed: bool) -> Result<(), ApiError> {
    if failed {
        return Err(ApiError::Transport);
    }
    if !retrieved {
        return Err(ApiError::InvalidResult);
    }
    Ok(())
}

#[cfg(test)]
#[path = "../tests/native/callbacks/tests.rs"]
mod tests;
