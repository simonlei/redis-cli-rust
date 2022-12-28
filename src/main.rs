extern crate core;

use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;

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

fn main() {
    let args = Args::parse();
    let redis_context = RedisContext {
        ip: args.host,
        port: args.port,
        password: args.password,
    };

    let stream = TcpStream::connect(redis_context.ip.to_string() + ":" + &redis_context.port.to_string()).unwrap();
    if redis_context.password.is_some() {
        write_and_get_result(&stream, "auth ".to_owned() + &redis_context.password.unwrap().to_string() + "\n");
    }
    loop {
        print!("{}:{}>", redis_context.ip, redis_context.port);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        // input = input[..input.len() - 1].to_string();
        if input.trim().eq("quit") { std::process::exit(0); }
        let response = write_and_get_result(&stream, input);
        println!("{response}");
    }
}

fn write_and_get_result(mut stream: &TcpStream, cmd: String) -> String {
    if cmd.trim().len() == 0 {
        return String::from("");
    }
    stream.write(cmd.as_bytes()).unwrap();
    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();

    if line.starts_with('$') {
        // first char is $x, then read x chars next line.
        response += &*read_next_line(line, &mut reader);
    } else if line.starts_with('*') {
        // first char is *x, then read 2x lines
        let sub = &line[1..].trim();
        let lines = sub.parse::<i32>().unwrap();
        for _ in 0..lines {
            let mut next_line = String::new();
            reader.read_line(&mut next_line).unwrap();
            // println!("next {next_line}");
            response += &*read_next_line(next_line, &mut reader);
            // should start with $
            // should read buffer
            // reader.read_line(&mut response).unwrap();
            // println!("2. {response}");
        }
    } else if line.starts_with(':') {
        let sub = &line[1..].trim();
        let lines = sub.parse::<i32>().unwrap();
        response = String::from("(integer) ") + &lines.to_string();
    } else {
        response = line;
    }


    response
}

fn read_next_line(line: String, reader: &mut BufReader<&TcpStream>) -> String {
    let sub = &line[1..].trim();
    // println!("sub is {sub}");
    let chars = sub.parse::<i32>().unwrap();

    if chars > 0 {
        let mut buf: Vec<u8> = vec![0; (chars + 2) as usize];
        reader.read_exact(buf.as_mut_slice()).unwrap();
        String::from_utf8_lossy(&*buf).to_string()
    } else {
        String::from("(nil)")
    }
}
