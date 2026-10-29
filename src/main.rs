mod args;
mod commands;

use std::process::ExitCode;
use std::process::exit;

use clap::Parser;

use crate::args::{Args, Command};
use crate::commands as cmd;

fn main() -> ExitCode {
    let args = Args::parse();
    let command = match args.command {
        Command::ProbeSteam => return commands::probe_steam(),
        Command::Inspect { app_id, item_id } => exit(cmd::inspect(app_id, item_id)),
        Command::UpdateFile { .. } => "update-file",
        Command::Download {
            app_id,
            item_id,
            output,
        } => exit(cmd::download(app_id, item_id, &output)),
    };

    eprintln!("error: {command} is not implemented yet");
    ExitCode::FAILURE
}
