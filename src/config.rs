pub mod conf {
    use dotenv::dotenv;
    use std::env;

    pub fn init() {
        dotenv().ok();
    }

    pub fn get_port() -> u16 {
        env::var("PORT")
            .unwrap_or("5005".to_string())
            .parse()
            .unwrap()
    }
    pub fn get_database_url() -> String {
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env")
    }

    pub fn get_redis_url() -> String {
        env::var("REDIS_URL").expect("REDIS_URL must be set in .env")
    }
}
