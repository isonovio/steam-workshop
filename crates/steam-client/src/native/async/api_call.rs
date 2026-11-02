use std::marker::PhantomData;
use std::num::NonZeroU64;

use crate::native::{ApiError, Runtime};

use super::callback::{Callback, Completion};

pub(crate) struct ApiCall<'runtime, T> {
    handle: NonZeroU64,
    callback_id: i32,
    result_size: usize,
    decode: fn(&[u8]) -> Result<T, ApiError>,
    _runtime: PhantomData<&'runtime Runtime>,
}

impl<'runtime, T> ApiCall<'runtime, T> {
    pub(in crate::native) fn new(
        handle: u64,
        callback_id: i32,
        result_size: usize,
        decode: fn(&[u8]) -> Result<T, ApiError>,
    ) -> Option<Self> {
        Some(Self {
            handle: NonZeroU64::new(handle)?,
            callback_id,
            result_size,
            decode,
            _runtime: PhantomData,
        })
    }

    pub(crate) fn resolve(&self, callback: &Callback<'_>) -> Result<Option<T>, ApiError> {
        let Some(completed) = callback.completion()? else {
            return Ok(None);
        };
        if !self.matches(&completed)? {
            return Ok(None);
        }
        let mut bytes = vec![0; self.result_size];
        callback.result(self.handle.get(), self.callback_id, &mut bytes)?;
        (self.decode)(&bytes).map(Some)
    }

    fn matches(&self, completed: &Completion) -> Result<bool, ApiError> {
        if completed.handle != self.handle.get() {
            return Ok(false);
        }
        if completed.callback_id != self.callback_id
            || self.result_size == 0
            || completed.size as usize != self.result_size
        {
            return Err(ApiError::InvalidResult);
        }
        Ok(true)
    }
}

#[cfg(test)]
#[path = "../../tests/native/async/api_call/tests.rs"]
mod tests;
