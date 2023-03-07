extern crate core;
extern crate redis;

use std::io;
use std::io::Write;

use atty::Stream;

use redis_context::RedisContext;

pub mod cmd_parser;
pub mod redis_context;
pub mod redis_funcs;

pub fn work_with_redis(mut redis_context: RedisContext) {
    loop {
        if atty::is(Stream::Stdin) {
            // only show prompt on tty
            print!("{}:{}> ", redis_context.ip, redis_context.port);
        }
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                if n == 0 {
                    // get eof, maybe pipeline
                    break;
                }
                if input.trim().eq("quit") {
                    break;
                }
                let response = redis_context.call_and_get_result(input);
                print!("{response}");
            }
            Err(err) => {
                println!("Error:{}, exit", err);
                break;
            }
        }
    }
}
