use std::time::Duration;

use crate::errors::Error;
use redis::Client;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub async fn connect_redis(uri: &str) -> Result<Client, Error> {
    let client = Client::open(uri).unwrap();
    Ok(client)
}

pub async fn connect_database(uri: &str) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(uri);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(false);
    Database::connect(opt).await.unwrap()
}
