use crate::native::ApiCall;
use crate::utils::Timeout;

use super::{CallError, Executor};

pub(crate) struct PendingCall<'a, 'runtime, T> {
    pub(crate) executor: &'a mut Executor<'runtime>,
    pub(crate) call: ApiCall<'runtime, T>,
}

impl<T> PendingCall<'_, '_, T> {
    /// A timeout stops waiting; it does not cancel the remote operation.
    pub fn wait(self, timeout: Timeout) -> Result<T, CallError> {
        self.executor.wait(&self.call, timeout)
    }
}
