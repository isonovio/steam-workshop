use std::num::NonZeroU64;

use crate::utils::Timeout;
use crate::{CallError, WorkshopItem};

use crate::steam::{PublishedFileDetails, PublishedFileVisibility};

use super::{Visibility, Workshop};

impl Workshop<'_> {
    pub fn inspect(
        &mut self,
        item_id: NonZeroU64,
        timeout: Timeout,
    ) -> Result<WorkshopItem, CallError> {
        let (steam, mut executor) = self.session.access();
        steam
            .remote_storage(|storage| {
                storage
                    .published_file(item_id)
                    .get_details(timeout, &mut executor)
                    .map(WorkshopItem::from)
            })
            .ok_or(CallError::InvalidResult)?
    }
}

impl From<PublishedFileDetails> for WorkshopItem {
    fn from(details: PublishedFileDetails) -> Self {
        Self {
            id: details.id,
            title: details.title,
            creator_app_id: details.creator_app_id,
            consumer_app_id: details.consumer_app_id,
            owner_id: details.owner_id,
            filename: details.filename,
            size: details.size,
            updated_at: details.updated_at,
            visibility: details.visibility.into(),
        }
    }
}

impl From<PublishedFileVisibility> for Visibility {
    fn from(visibility: PublishedFileVisibility) -> Self {
        match visibility {
            PublishedFileVisibility::Public => Self::Public,
            PublishedFileVisibility::FriendsOnly => Self::FriendsOnly,
            PublishedFileVisibility::Private => Self::Private,
            PublishedFileVisibility::Unlisted => Self::Unlisted,
        }
    }
}

#[cfg(test)]
#[path = "../tests/workshop/inspect/tests.rs"]
mod tests;
