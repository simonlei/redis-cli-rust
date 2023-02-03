extern crate core;
extern crate redis;

use std::io;
use std::io::Write;

use atty::Stream;
use clap::{Parser, Subcommand};
use derivative::Derivative;
use redis::{Connection, ConnectionLike, from_redis_value, Value};
use shellwords;

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

fn main() {
    let args = Args::parse();
    let redis_context = RedisContext {
        ip: args.host,
        port: args.port,
        password: args.password,
    };

    let (redis_context, mut con) = make_connection(redis_context).unwrap();

    loop {
        if atty::is(Stream::Stdin) {
            // only show prompt on tty
            print!("{}:{}> ", redis_context.ip, redis_context.port);
        }
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                if n == 0 { // get eof, maybe pipeline
                    break;
                }
                if input.trim().eq("quit") { break; }
                let response = call_and_get_result(&mut con, input);
                print!("{response}");
            }
            Err(err) => {
                println!("Error:{}, exit", err);
                break;
            }
        }
    }
}

fn make_connection(redis_context: RedisContext) -> redis::RedisResult<(RedisContext, Connection)> {
    let conn_string = redis_context.get_connection_string();
    // println!("{}", conn_string);
    let client = redis::Client::open(conn_string)?;
    let con = client.get_connection()?;
    Ok((redis_context, con))
}

fn call_and_get_result(con: &mut redis::Connection, input: String) -> String {
    if input.trim().len() == 0 {
        return String::from("");
    }

    let cmds: Vec<String> = shellwords::split(input.trim()).unwrap();
    let args = &cmds[1..];
    let mut cmd = redis::cmd(cmds.get(0).unwrap());
    for arg in args {
        cmd.arg(arg);
    }

    let value = match con.req_command(&cmd) {
        Ok(v) => v,
        Err(err) => Value::Status(format!("(error) {}", err)),
    };
    match value {
        Value::Nil => String::from("(nil)\n"),
        Value::Data(data) => format!("\"{}\"\n", String::from_utf8_lossy(data.as_slice())),
        Value::Bulk(bulk) => format_bulk_data(bulk),
        Value::Int(data) => format!("(integer) {}\n", data),
        Value::Status(status) => status + "\n",
        Value::Okay => String::from("OK\n"),
    }
}

fn format_bulk_data(bulk: Vec<Value>) -> String {
    let size = bulk.len();
    if size == 0 {
        return String::from("(empty list or set)");
    }
    let mut result = String::new();
    let mut i = 1;
    let col = (size.ilog10() + 1) as usize;
    for data in bulk {
        let str: String = match data {
            Value::Data(d) => match String::from_utf8(d.clone()) {
                Ok(str) => str,
                Err(_) => format_vec_with_unicode(d),
            },
            _ => from_redis_value(&data).unwrap(),
        };
        result += &format!("{:>col$}) \"{}\"\n", i, str);
        i += 1;
    }
    result
}

fn format_vec_with_unicode(data: Vec<u8>) -> String {
    let mut result = String::new();
    for c in data {
        if c.is_ascii() && !c.is_ascii_control() {
            result += &format!("{}", c as char).to_string();
        } else {
            result += &format!("{:#04x}", c as usize).to_string().replace("0x", "\\x");
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get() -> redis::RedisResult<()> {
        let (redis_context, mut con) = make_connection(RedisContext::default())?;
        assert_eq!(redis_context.port, 6379);
        call_and_get_result(&mut con, String::from("set c 1"));
        assert_eq!("\"1\"\n", call_and_get_result(&mut con, String::from("get c")));
        call_and_get_result(&mut con, String::from("set c '1 2 3'"));
        assert_eq!("\"1 2 3\"\n", call_and_get_result(&mut con, String::from("get c")));
        call_and_get_result(&mut con, String::from("set \"c d\" '1 2 3'"));
        assert_eq!("\"1 2 3\"\n", call_and_get_result(&mut con, String::from("get \"c d\"")));
        return Ok(());
    }

    #[test]
    fn test_keys() -> redis::RedisResult<()> {
        let (_, mut con) = make_connection(RedisContext::default())?;
        call_and_get_result(&mut con, String::from("set c 1"));
        assert_eq!("(empty list or set)", call_and_get_result(&mut con, String::from("keys key_not_exist")));
        assert_eq!("1) \"c\"\n", call_and_get_result(&mut con, String::from("keys c")));
        return Ok(());
    }

    #[test]
    #[ignore]
    fn test_all_keys() -> redis::RedisResult<()> {
        let (_, mut con) = make_connection(RedisContext {
            ip: String::from("xxx"),
            port: 6380,
            password: Some(String::from("xxx")),
        })?;
        call_and_get_result(&mut con, String::from("keys *"));
        return Ok(());
    }
}
