extern crate core;

use std::{env, io};
use std::io::Write;

struct RedisContext {
    ip: String,
    port: u16,
    auth: String,
}

fn main() {
    // TODO: parse command line to get redis context
    let redis_context = RedisContext {
        ip: String::from("9.134.105.13"),
        port: 6380,
        auth: env::var("AUTH").unwrap(),
    };

    loop {
        print!("{}:{}>", redis_context.ip, redis_context.port);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input[..input.len() - 1].to_string();
        if input.eq("quit") { std::process::exit(0); }
        println!("Hello, world! {input}");
    }
}
