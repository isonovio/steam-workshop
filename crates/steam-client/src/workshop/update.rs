use std::num::NonZeroU64;
use std::time::Duration;

use thiserror::Error;

use crate::{CallError, MAX_UPLOAD_SIZE, WorkshopItem};

use super::Workshop;

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

impl Workshop<'_> {
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
