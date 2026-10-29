use std::num::NonZeroU64;
use std::time::{Duration, Instant};

use crate::{CallError, Session};

impl Session {
    pub fn download(
        &mut self,
        item_id: NonZeroU64,
        timeout: Duration,
    ) -> Result<Vec<u8>, CallError> {
        let start = Instant::now();
        let item = self.inspect(item_id, timeout)?;
        self.remote_storage()
            .ok_or(CallError::InvalidResult)?
            .download_content(
                item.content_handle,
                item.size,
                timeout.saturating_sub(start.elapsed()),
            )
    }
}
