use redis_cli_rust::cmd_parser;

fn main() {
    let redis_context = cmd_parser::parse_args();
    redis_cli_rust::work_with_redis(redis_context)
}
