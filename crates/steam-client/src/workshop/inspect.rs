use std::num::NonZeroU64;
use std::time::Duration;

use crate::{CallError, WorkshopItem};

use super::Workshop;

impl Workshop<'_> {
    pub fn inspect(
        &mut self,
        item_id: NonZeroU64,
        timeout: Duration,
    ) -> Result<WorkshopItem, CallError> {
        self.session
            .remote_storage()
            .ok_or(CallError::InvalidResult)?
            .published_file_details(item_id, timeout)
    }
}
