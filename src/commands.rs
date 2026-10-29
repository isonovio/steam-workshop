use std::num::{NonZeroU32, NonZeroU64};
use std::process::ExitCode;
use std::time::Duration;

pub fn probe_steam() -> ExitCode {
    match steam_client::is_running() {
        true => {
            println!("Steam is running");
            ExitCode::SUCCESS
        }
        false => {
            println!("Steam is not running");
            ExitCode::FAILURE
        }
    }
}

pub fn inspect(app_id: NonZeroU32, item_id: NonZeroU64) -> i32 {
    match steam_client::run(app_id, |session| {
        let item = session.inspect(item_id, Duration::from_secs(30))?;
        println!("Item: {}", item.id);
        println!("Title: {:?}", item.title);
        println!("Creator app: {}", item.creator_app_id);
        println!("Consumer app: {}", item.consumer_app_id);
        println!("Owner: {}", item.owner_id);
        println!("Visibility: {:?}", item.visibility);
        println!("File: {:?}", item.filename);
        println!("Size: {} bytes", item.size);
        println!("Updated: {} (Unix seconds)", item.updated_at);
        Ok::<(), steam_client::CallError>(())
    }) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            1
        }
    }
}
