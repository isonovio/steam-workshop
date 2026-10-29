mod args;
mod commands;

use std::process::ExitCode;
use std::process::exit;

use clap::Parser;

use crate::args::{Args, Command};
use crate::commands as cmd;

fn main() -> ExitCode {
    let args = Args::parse();
    match args.command {
        Command::ProbeSteam => cmd::probe_steam(),
        Command::Inspect { app_id, item_id } => exit(cmd::inspect(app_id, item_id)),
        Command::UpdateFile {
            app_id,
            item_id,
            file,
        } => exit(cmd::update_file(app_id, item_id, &file)),
        Command::Download {
            app_id,
            item_id,
            output,
        } => exit(cmd::download(app_id, item_id, &output)),
    }
}
