use std::ffi::CString;
use std::num::NonZeroU64;
use std::time::{Duration, Instant};

use thiserror::Error;

use crate::{CallError, CloudError, CloudOperation, MAX_UPLOAD_SIZE, RemoteStorage, WorkshopItem};

use crate::steam::{Executor, PendingCall};

use crate::features::{CommitResult, PublishedUpdate};

use super::{UpdateResult, Workshop};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UpdateError {
    #[error("Input exceeds the {MAX_UPLOAD_SIZE}-byte upload limit")]
    TooLarge,
    #[error("Item belongs to app {actual}, expected {expected}")]
    WrongApp { expected: u32, actual: u32 },
    #[error("Item is not owned by the logged-in user")]
    NotOwner,
    #[error(transparent)]
    Call(#[from] CallError),
    #[error(transparent)]
    Cloud(#[from] CloudError),
}

impl Workshop<'_> {
    pub fn update_file(
        &mut self,
        item_id: NonZeroU64,
        bytes: &[u8],
        timeout: Duration,
    ) -> Result<UpdateResult, UpdateError> {
        let start = Instant::now();
        self.validate_update(item_id, bytes, timeout)?;
        let needs_legal_agreement =
            self.with_temporary_cloud_file(bytes, |storage, filename, executor| {
                let remaining = timeout.saturating_sub(start.elapsed());
                if remaining.is_zero() {
                    return CloudOperation::Complete(Err(CallError::Timeout));
                }
                let call = match submit(storage, item_id, filename, executor) {
                    Ok(call) => call,
                    Err(error) => return CloudOperation::Complete(Err(error)),
                };
                completion(call.wait(remaining), item_id)
            })?;
        Ok(UpdateResult {
            item_id: item_id.get(),
            needs_legal_agreement,
        })
    }

    pub fn validate_update(
        &mut self,
        item_id: NonZeroU64,
        bytes: &[u8],
        timeout: Duration,
    ) -> Result<WorkshopItem, UpdateError> {
        if bytes.len() > MAX_UPLOAD_SIZE {
            return Err(UpdateError::TooLarge);
        }
        let item = self.inspect(item_id, timeout)?;
        item.validate_update(self.session.app_id(), self.session.user_id())?;
        Ok(item)
    }
}

fn submit<'a, 'session>(
    storage: RemoteStorage<'session>,
    item_id: NonZeroU64,
    filename: &str,
    executor: &'a mut Executor<'session>,
) -> Result<PendingCall<'a, 'session, CommitResult>, CallError> {
    let filename = CString::new(filename).map_err(|_| CallError::InvalidResult)?;
    let mut update = PublishedUpdate::new(storage, item_id).ok_or(CallError::InvalidHandle)?;
    if !update.set_file(&filename) {
        return Err(CallError::InvalidResult);
    }
    update.commit(executor).ok_or(CallError::InvalidHandle)
}

fn completion(
    result: Result<CommitResult, CallError>,
    item_id: NonZeroU64,
) -> CloudOperation<bool> {
    match result {
        Ok(result) if result.item_id == item_id.get() => {
            CloudOperation::Complete(result.result.map_err(CallError::from))
        }
        Ok(_) => CloudOperation::Uncertain(CallError::InvalidResult),
        Err(error) => CloudOperation::Uncertain(error),
    }
}

impl WorkshopItem {
    fn validate_update(&self, app_id: u32, user_id: u64) -> Result<(), UpdateError> {
        if self.consumer_app_id != app_id {
            return Err(UpdateError::WrongApp {
                expected: app_id,
                actual: self.consumer_app_id,
            });
        }
        if self.owner_id != user_id {
            return Err(UpdateError::NotOwner);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../tests/workshop/update/tests.rs"]
mod tests;
