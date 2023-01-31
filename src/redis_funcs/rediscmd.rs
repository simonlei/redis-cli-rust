use clap::{Parser, Subcommand};
use derivative::Derivative;

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "redisclient")]
pub(crate) struct RedisCmds {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Set {
        key: String,
        value: String,
    }
}