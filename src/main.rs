mod args;
mod commands;

use std::process::ExitCode;

use clap::Parser;

use crate::args::{Args, Command};

fn main() -> ExitCode {
    let args = Args::parse();
    let command = match args.command {
        Command::ProbeSteam => return commands::probe_steam(),
        Command::Inspect { app_id, item_id } => {
            let code = commands::inspect(app_id, item_id);
            std::process::exit(code);
        }
        Command::UpdateFile { .. } => "update-file",
        Command::Download { .. } => "download",
    };

    eprintln!("error: {command} is not implemented yet");
    ExitCode::FAILURE
}
