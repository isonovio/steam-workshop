use std::num::NonZeroU64;
use std::time::Duration;

use crate::CallError;

use super::Workshop;

impl Workshop<'_> {
    pub fn download(
        &mut self,
        item_id: NonZeroU64,
        timeout: Duration,
    ) -> Result<Vec<u8>, CallError> {
        let (steam, mut executor) = self.session.access();
        steam
            .remote_storage(|storage| {
                let published_file = storage.published_file(item_id);
                published_file.download(&mut executor, timeout)
            })
            .ok_or(CallError::InvalidResult)?
    }
}
