use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Default)]
pub struct RedisContext {
    #[derivative(Default(value = "String::from(\"127.0.0.1\")"))]
    pub ip: String,
    #[derivative(Default(value = "6379"))]
    pub port: u16,
    pub password: Option<String>,
}

impl RedisContext {
    pub(crate) fn get_connection_string(&self) -> String {
        let auth = match &self.password {
            None => String::from(""),
            Some(str) => format!(":{str}@"),
        };
        format!("redis://{auth}{0}:{1}", self.ip, self.port)
    }
}
