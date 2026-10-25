use std::num::{NonZeroU32, NonZeroU64};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "CLI for interacting with the Steam Workshop")]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Inspect a Workshop item")]
    Inspect {
        app_id: NonZeroU32,
        item_id: NonZeroU64,
    },
    #[command(about = "Update a Workshop item's file")]
    UpdateFile {
        app_id: NonZeroU32,
        item_id: NonZeroU64,
        file: PathBuf,
    },
    #[command(about = "Download a Workshop item's file")]
    Download {
        app_id: NonZeroU32,
        item_id: NonZeroU64,
        output: PathBuf,
    },
}
