extern crate core;
extern crate redis;

use std::io;
use std::io::Write;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(disable_help_flag = true)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("127.0.0.1"))]
    host: String,
    #[arg(short, long, default_value_t = 6379)]
    port: u16,
    #[arg(short = 'a', long)]
    password: Option<String>,
    #[clap(subcommand)]
    cmd: Option<Command>,
}

#[derive(Subcommand)]
enum Command {}

struct RedisContext {
    ip: String,
    port: u16,
    password: Option<String>,
}

impl RedisContext {
    fn get_connection_string(&self) -> String {
        let auth = match &self.password {
            None => String::from(""),
            Some(str) => format!(":{str}@"),
        };
        format!("redis://{auth}{0}:{1}", self.ip, self.port)
    }
}

fn main() -> redis::RedisResult<()> {
    let args = Args::parse();
    let redis_context = RedisContext {
        ip: args.host,
        port: args.port,
        password: args.password,
    };
    let conn_string = redis_context.get_connection_string();
    println!("{}", conn_string);
    let client = redis::Client::open(conn_string)?;
    let mut con = client.get_connection()?;

    loop {
        print!("{}:{}>", redis_context.ip, redis_context.port);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim().eq("quit") { std::process::exit(0); }
        let response = call_and_get_result(&mut con, input);
        println!("{response}");
    }
}

fn call_and_get_result(con: &mut redis::Connection, input: String) -> String {
    if input.trim().len() == 0 {
        return String::from("");
    }
    let cmds: Vec<&str> = input.trim().split_whitespace().collect();
    let args = &cmds[1..];
    let mut cmd = redis::cmd(cmds.get(0).unwrap());
    for arg in args {
        cmd.arg(arg);
    }
    cmd.query(con).unwrap()
}
