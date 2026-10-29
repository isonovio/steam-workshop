use std::num::{NonZeroU32, NonZeroU64};
use std::time::Duration;

use steam_client::{CallError, Session, WorkshopItem};

use super::call_client;

pub fn command(app_id: NonZeroU32, item_id: NonZeroU64) -> i32 {
    call_client(app_id, |session| inspect(session, item_id))
}

fn inspect(session: &mut Session, item_id: NonZeroU64) -> Result<(), CallError> {
    let item = session.inspect(item_id, Duration::from_secs(30))?;
    show(&item);
    Ok(())
}

fn show(item: &WorkshopItem) {
    println!("Item: {}", item.id);
    println!("Title: {:?}", item.title);
    println!("Creator app: {}", item.creator_app_id);
    println!("Consumer app: {}", item.consumer_app_id);
    println!("Owner: {}", item.owner_id);
    println!("Visibility: {:?}", item.visibility);
    println!("File: {:?}", item.filename);
    println!("Size: {} bytes", item.size);
    println!("Updated: {} (Unix seconds)", item.updated_at);
}
