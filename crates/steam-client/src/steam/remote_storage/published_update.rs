use std::num::NonZeroU64;
use std::time::{Duration, Instant};

use crate::native::ApiCall;
use crate::native::results::CommitResult;

use crate::steam::{CallError, Executor, PendingCall};

use super::{CloudFile, PublishedFile};

struct UpdateRejected<'runtime, 'storage> {
    file: CloudFile<'runtime, 'storage>,
    error: CallError,
}

#[must_use = "Dropping a pending update retains its Cloud file without cancelling the update"]
struct PendingUpdate<'executor, 'runtime, 'storage> {
    call: PendingCall<'executor, 'runtime, CommitResult>,
    file: CloudFile<'runtime, 'storage>,
    item: PublishedFile<'runtime, 'storage>,
}

pub enum UpdateOutcome<'runtime, 'storage> {
    NotSubmitted {
        file: CloudFile<'runtime, 'storage>,
        error: CallError,
    },
    Complete {
        file: CloudFile<'runtime, 'storage>,
        result: Result<bool, CallError>,
    },
    Uncertain {
        filename: String,
        error: CallError,
    },
}

enum Completion {
    Confirmed(Result<bool, CallError>),
    Uncertain(CallError),
}

impl<'runtime, 'storage> PublishedFile<'runtime, 'storage> {
    /// Performs submission and waiting as one operation; timeout does not cancel Steam's work.
    pub fn update_with(
        self,
        file: CloudFile<'runtime, 'storage>,
        executor: &mut Executor<'runtime>,
        timeout: Duration,
    ) -> UpdateOutcome<'runtime, 'storage> {
        let start = Instant::now();
        if timeout.is_zero() {
            return UpdateOutcome::NotSubmitted {
                file,
                error: CallError::Timeout,
            };
        }
        let call = self.submit(&file);
        match PendingUpdate::new(self, file, executor, call) {
            Ok(pending) => pending.wait(timeout.saturating_sub(start.elapsed())),
            Err(rejected) => UpdateOutcome::NotSubmitted {
                file: rejected.file,
                error: rejected.error,
            },
        }
    }

    fn submit(
        &self,
        file: &CloudFile<'runtime, 'storage>,
    ) -> Result<ApiCall<'runtime, CommitResult>, CallError> {
        let mut update = self.native.update().ok_or(CallError::InvalidHandle)?;
        if !update.set_file(file.native.name()) {
            return Err(CallError::InvalidResult);
        }
        update.commit().ok_or(CallError::InvalidHandle)
    }
}

impl<'executor, 'runtime, 'storage> PendingUpdate<'executor, 'runtime, 'storage> {
    fn new(
        item: PublishedFile<'runtime, 'storage>,
        file: CloudFile<'runtime, 'storage>,
        executor: &'executor mut Executor<'runtime>,
        call: Result<ApiCall<'runtime, CommitResult>, CallError>,
    ) -> Result<Self, UpdateRejected<'runtime, 'storage>> {
        match call {
            Ok(call) => Ok(Self {
                call: PendingCall { executor, call },
                file,
                item,
            }),
            Err(error) => Err(UpdateRejected { file, error }),
        }
    }
}

impl<'runtime, 'storage> PendingUpdate<'_, 'runtime, 'storage> {
    fn wait(self, timeout: Duration) -> UpdateOutcome<'runtime, 'storage> {
        resolve(self.file, self.call.wait(timeout), self.item.item_id())
    }
}

fn resolve<'runtime, 'storage>(
    file: CloudFile<'runtime, 'storage>,
    result: Result<CommitResult, CallError>,
    item_id: NonZeroU64,
) -> UpdateOutcome<'runtime, 'storage> {
    match completion(result, item_id) {
        Completion::Confirmed(result) => UpdateOutcome::Complete { file, result },
        Completion::Uncertain(error) => UpdateOutcome::Uncertain {
            filename: file.retain(),
            error,
        },
    }
}

fn completion(result: Result<CommitResult, CallError>, item_id: NonZeroU64) -> Completion {
    match result {
        Ok(result) => match result.item_id == item_id.get() {
            true => Completion::Confirmed(result.result.map_err(CallError::from)),
            false => Completion::Uncertain(CallError::InvalidResult),
        },
        Err(error) => Completion::Uncertain(error),
    }
}

#[cfg(test)]
#[path = "../../tests/steam/remote_storage/published_update/tests.rs"]
mod tests;
