extern crate core;
extern crate redis;

use std::io;
use std::io::Write;

use atty::Stream;
use redis::{Connection, ConnectionLike, from_redis_value, Value};
use regex::Regex;

use redis_context::RedisContext;

pub mod redis_funcs;
pub mod cmd_parser;
pub mod redis_context;

pub fn work_with_redis(redis_context: RedisContext, con: &mut Connection) {
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
                let response = call_and_get_result(con, input);
                print!("{response}");
            }
            Err(err) => {
                println!("Error:{}, exit", err);
                break;
            }
        }
    }
}


pub fn make_connection(redis_context: &RedisContext) -> redis::RedisResult<Connection> {
    let conn_string = redis_context.get_connection_string();
    // println!("{}", conn_string);
    let client = redis::Client::open(conn_string)?;
    let con = client.get_connection()?;
    Ok(con)
}

fn call_and_get_result(con: &mut redis::Connection, input: String) -> String {
    if input.trim().is_empty() {
        return String::from("");
    }

    let cmds: Vec<String> = shell_words::split(unescape_unicode(&input).trim()).unwrap();
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

fn unescape_unicode(str: &str) -> String {
    // println!("To unescape:{}", str);
    let re = Regex::new(r"\\x([0-9 a-f][0-9 a-f])").unwrap();
    let mut locations = re.capture_locations();
    let mut loc = 0;
    let mut bytes: Vec<u8> = Vec::new();
    while re.captures_read_at(&mut locations, str, loc).is_some() {
        let (start, end) = locations.get(0).unwrap();
        if start > loc {
            bytes.append(&mut str.get(loc..start).unwrap().as_bytes().to_vec());
        }
        if start > 0 && str.get(start - 1..start).unwrap().eq("\\") {
            // it's \\xaa, should not be escaped.
            bytes.append(&mut str.get(start..end).unwrap().as_bytes().to_vec());
        } else {
            bytes.push(u8::from_str_radix(str.get(start + 2..end).unwrap(), 16).unwrap());
            // println!("From {} to {}", start, end);
        }
        loc = end;
    }
    bytes.append(&mut str.get(loc..).unwrap().as_bytes().to_vec());

    String::from_utf8(bytes).unwrap()
}

#[cfg(test)]
mod tests {
    use redis::Commands;

    use super::*;

    #[test]
    fn test_set_get() -> redis::RedisResult<()> {
        let redis_context = RedisContext::default();
        let mut con = make_connection(&redis_context)?;
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
        let mut con = make_connection(&RedisContext::default())?;
        call_and_get_result(&mut con, String::from("set c 1"));
        assert_eq!("(empty list or set)", call_and_get_result(&mut con, String::from("keys key_not_exist")));
        assert_eq!("1) \"c\"\n", call_and_get_result(&mut con, String::from("keys c")));
        return Ok(());
    }

    #[test]
    fn test_unicode_keys() -> redis::RedisResult<()> {
        assert_eq!("中文key", String::from_utf8(b"\xe4\xb8\xad\xe6\x96\x87key".to_vec()).unwrap());
        let mut con = make_connection(&RedisContext::default())?;
        call_and_get_result(&mut con, String::from("set 中文key 1"));
        let result: Option<String> = con.get(b"\xe4\xb8\xad\xe6\x96\x87key")?;
        assert_eq!("1", result.unwrap());
        call_and_get_result(&mut con, String::from("set 中文key 1"));
        assert_eq!("\"1\"\n", call_and_get_result(&mut con, String::from("get 中文key")));
        assert_eq!("\"1\"\n", call_and_get_result(&mut con, String::from_utf8(b"get '\xe4\xb8\xad\xe6\x96\x87key'".to_vec()).unwrap()));
        assert_eq!("\"1\"\n", call_and_get_result(&mut con, String::from("get '\\xe4\\xb8\\xad\\xe6\\x96\\x87key'")));

        call_and_get_result(&mut con, String::from("set \"\\\\xe4\\\\xb8\\\\xad\" noquote"));
        assert_eq!("\"noquote\"\n", call_and_get_result(&mut con, String::from("get \"\\\\xe4\\\\xb8\\\\xad\"")));

        Ok(())
    }

    #[test]
    fn test_unescape_unicode() {
        assert_eq!("\\\\xe4\\\\xb8\\\\xad", unescape_unicode(&String::from("\\\\xe4\\\\xb8\\\\xad")));
        assert_eq!("中文S", unescape_unicode(&String::from("\\xe4\\xb8\\xad\\xe6\\x96\\x87\\x53")));
        assert_eq!("中文", unescape_unicode(&String::from("\\xe4\\xb8\\xad\\xe6\\x96\\x87")));
        assert_eq!("x中文", unescape_unicode(&String::from("x\\xe4\\xb8\\xad\\xe6\\x96\\x87")));
        assert_eq!("中文key", unescape_unicode(&String::from("\\xe4\\xb8\\xad\\xe6\\x96\\x87key")));
        assert_eq!("中x文key", unescape_unicode(&String::from("\\xe4\\xb8\\xadx\\xe6\\x96\\x87key")));
        assert_eq!("y中x文key", unescape_unicode(&String::from("y\\xe4\\xb8\\xadx\\xe6\\x96\\x87key")));
    }

    #[test]
    #[ignore]
    fn test_all_keys() -> redis::RedisResult<()> {
        let mut con = make_connection(&RedisContext {
            ip: String::from("xxx"),
            port: 6380,
            password: Some(String::from("xxx")),
        })?;
        call_and_get_result(&mut con, String::from("keys *"));
        return Ok(());
    }
}
