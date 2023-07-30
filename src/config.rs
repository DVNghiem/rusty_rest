pub struct Config {
    host: String,
    port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        let host = std::env::var("HOST").unwrap_or("localost".to_string()).to_string();
        let port: u16 = match std::env::var("PORT") {
            Ok(val) => val.parse().unwrap_or(5005),
            Err(err) => panic!("Error PORT env {:?}", err),
        };
        Self { host, port }
    }

    pub fn get_host(&self) -> &String {
        &self.host
    }

    pub fn get_port(self) -> u16 {
        self.port
    }
}
