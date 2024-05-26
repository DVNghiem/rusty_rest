use dotenv::dotenv;
use std::env;

pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
}

impl Config {
    pub fn new() -> Self {
        dotenv().ok();
        Self {
            port: env::var("PORT")
                .unwrap_or("5005".to_string())
                .parse()
                .unwrap(),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env"),
            redis_url: env::var("REDIS_URL").expect("REDIS_URL must be set in .env"),
        }
    }
}
