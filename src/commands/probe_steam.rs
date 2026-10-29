use std::process::ExitCode;

pub fn command() -> ExitCode {
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
