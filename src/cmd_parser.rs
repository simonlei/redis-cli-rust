use clap::{Parser, Subcommand};

use super::redis_context;

#[derive(Parser)]
#[clap(disable_help_flag = true)]
#[command(author, version, about)]
pub struct Args {
    #[arg(short, long, default_value = "127.0.0.1")]
    pub host: String,
    #[arg(short, long, default_value_t = 6379)]
    pub port: u16,
    #[arg(short = 'a', long)]
    pub password: Option<String>,
    #[clap(subcommand)]
    cmd: Option<Command>,
    #[arg(long, action = clap::ArgAction::Help)]
    help: Option<bool>,
}

#[derive(Subcommand)]
enum Command {}

pub fn parse_args() -> redis_context::RedisContext {
    let args = Args::parse();
    redis_context::RedisContext {
        ip: args.host,
        port: args.port,
        password: args.password,
        con: None,
    }
}
