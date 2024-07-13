use dotenv::dotenv;
use std::env;

pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
    pub aws_region: String,
    pub aws_bucket: String,
}

impl Config {
    pub fn new() -> Self {
        dotenv().ok();
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env"),
            redis_url: env::var("REDIS_URL").expect("REDIS_URL must be set in .env"),
            aws_access_key_id: env::var("AWS_ACCESS_KEY_ID").unwrap_or_default(),
            aws_secret_access_key: env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_default(),
            aws_region: env::var("AWS_REGION").unwrap_or_default(),
            aws_bucket: env::var("AWS_BUCKET").unwrap_or_default(),
        }
    }
}
