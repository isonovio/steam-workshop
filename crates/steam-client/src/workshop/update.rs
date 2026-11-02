use std::num::NonZeroU64;

use thiserror::Error;

use crate::utils::Timeout;
use crate::{CallError, CloudError, CloudFile, MAX_UPLOAD_SIZE, WorkshopItem};

use super::staging::{finish, temporary_filename};
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
        timeout: Timeout,
    ) -> Result<UpdateResult, UpdateError> {
        self.validate_update(item_id, bytes, timeout)?;
        let timeout = timeout.check().ok_or(CallError::Timeout)?;
        let (steam, mut executor) = self.session.access();
        let needs_legal_agreement = steam
            .remote_storage(|storage| {
                let file = CloudFile::new(storage, &temporary_filename()?, bytes)?;
                let published_file = storage.published_file(item_id);
                let outcome = published_file.update_with(file, &mut executor, timeout);
                finish(outcome)
            })
            .ok_or(CloudError::Unavailable)??;
        Ok(UpdateResult {
            item_id: item_id.get(),
            needs_legal_agreement,
        })
    }

    pub fn validate_update(
        &mut self,
        item_id: NonZeroU64,
        bytes: &[u8],
        timeout: Timeout,
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
