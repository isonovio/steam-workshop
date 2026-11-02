use std::time::Duration;

use crate::native::ApiCall;

use super::{CallError, Executor};

pub(crate) struct PendingCall<'a, 'runtime, T> {
    pub(crate) executor: &'a mut Executor<'runtime>,
    pub(crate) call: ApiCall<'runtime, T>,
}

impl<T> PendingCall<'_, '_, T> {
    /// A timeout stops waiting; it does not cancel the remote operation.
    pub fn wait(self, timeout: Duration) -> Result<T, CallError> {
        self.executor.wait(&self.call, timeout)
    }
}
