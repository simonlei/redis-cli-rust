extern crate core;

use std::{env, io};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;

struct RedisContext {
    ip: String,
    port: u16,
    auth: String,
}

fn main() {
    // TODO: parse command line to get redis context
    let redis_context = RedisContext {
        ip: env::var("IP").unwrap(),
        port: env::var("PORT").unwrap().parse::<u16>().unwrap(),
        auth: env::var("AUTH").unwrap(),
    };


    let mut stream = TcpStream::connect(redis_context.ip.to_string() + ":" + &redis_context.port.to_string()).unwrap();
    write_and_get_result(&stream, "auth ".to_owned() + &redis_context.auth.to_string() + "\n");

    loop {
        print!("{}:{}>", redis_context.ip, redis_context.port);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        // input = input[..input.len() - 1].to_string();
        if input.eq("quit") { std::process::exit(0); }
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
        for i in 0..lines {
            let mut next_line = String::new();
            reader.read_line(&mut next_line).unwrap();
            println!("next {next_line}");
            response += &*read_next_line(next_line, &mut reader);
            // should start with $
            // should read buffer
            // reader.read_line(&mut response).unwrap();
            // println!("2. {response}");
        }
    } else {
        response = line;
    }


    response
}

fn read_next_line(line: String, reader: &mut BufReader<&TcpStream>) -> String {
    let sub = &line[1..].trim();
    println!("sub is {sub}");
    let chars = sub.parse::<i32>().unwrap();

    if chars > 0 {
        let mut buf: Vec<u8> = vec![0; (chars + 2) as usize];
        reader.read_exact(buf.as_mut_slice()).unwrap();
        String::from_utf8(buf).unwrap()
    } else {
        String::from("(nil)")
    }
}
