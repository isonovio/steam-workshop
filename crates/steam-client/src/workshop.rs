use std::num::NonZeroU64;
use std::time::Duration;

use crate::{CallError, Session, WorkshopItem};

impl Session {
    pub fn inspect(
        &mut self,
        item_id: NonZeroU64,
        timeout: Duration,
    ) -> Result<WorkshopItem, CallError> {
        self.remote_storage()
            .ok_or(CallError::InvalidResult)?
            .published_file_details(item_id, timeout)
    }
}
