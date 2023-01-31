extern crate core;
extern crate redis;

use std::io;
use std::io::Write;

use clap::{Parser, Subcommand};
use derivative::Derivative;
use redis::Connection;
use shellwords;

use redis_funcs::rediscmd::RedisCmds;

pub mod redis_funcs;

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

#[derive(Derivative)]
#[derivative(Default)]
struct RedisContext {
    #[derivative(Default(value = "String::from(\"127.0.0.1\")"))]
    ip: String,
    #[derivative(Default(value = "6379"))]
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

    let (redis_context, mut con) = make_connection(redis_context)?;

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

fn make_connection(redis_context: RedisContext) -> redis::RedisResult<(RedisContext, Connection)> {
    let conn_string = redis_context.get_connection_string();
    println!("{}", conn_string);
    let client = redis::Client::open(conn_string)?;
    let con = client.get_connection()?;
    Ok((redis_context, con))
}

fn call_and_get_result(con: &mut redis::Connection, input: String) -> String {
    if input.trim().len() == 0 {
        return String::from("");
    }
    // RedisCmds::parse(input);

    let cmds: Vec<String> = shellwords::split(input.trim()).unwrap();
    //input.trim().split_whitespace().collect();
    let args = &cmds[1..];
    let mut cmd = redis::cmd(cmds.get(0).unwrap());
    for arg in args {
        cmd.arg(arg);
    }
    cmd.query(con).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::{call_and_get_result, make_connection, RedisContext};

    #[test]
    fn test_make_connection() -> redis::RedisResult<()> {
        let (redis_context, mut con) = make_connection(RedisContext::default())?;
        assert_eq!(redis_context.port, 6379);
        call_and_get_result(&mut con, String::from("set c 1"));
        assert_eq!("1", call_and_get_result(&mut con, String::from("get c")));
        call_and_get_result(&mut con, String::from("set c '1 2 3'"));
        assert_eq!("1 2 3", call_and_get_result(&mut con, String::from("get c")));
        call_and_get_result(&mut con, String::from("set \"c d\" '1 2 3'"));
        assert_eq!("1 2 3", call_and_get_result(&mut con, String::from("get \"c d\"")));
        return Ok(());
    }

    #[test]
    fn test_nothing() {}
}
