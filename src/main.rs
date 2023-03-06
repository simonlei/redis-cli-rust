use redis_cli_rust::cmd_parser;

fn main() {
    let redis_context = cmd_parser::parse_args();
    let mut con = redis_cli_rust::make_connection(&redis_context).unwrap();

    redis_cli_rust::work_with_redis(redis_context, &mut con)
}
