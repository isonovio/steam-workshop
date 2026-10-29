use std::num::NonZeroU64;
use std::time::Duration;

use steamworks::sys;
use thiserror::Error;

use crate::{CallError, Session, WorkshopItem};

pub const MAX_UPLOAD_SIZE: usize = sys::k_unMaxCloudFileChunkSize as usize;

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
}

impl Session {
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
        item.validate_update(self.app_id(), self.user_id())?;
        Ok(item)
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
#[path = "tests/update/tests.rs"]
mod tests;
