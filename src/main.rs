fn main() {
    let redis_context = redis_cli_rust::parse_args();
    let (redis_context, mut con) = redis_cli_rust::make_connection(redis_context).unwrap();

    redis_cli_rust::work_with_redis(redis_context, &mut con)
}
