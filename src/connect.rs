use std::time::Duration;

use crate::{config::conf, errors::Error};
use redis::Client;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tokio::sync::OnceCell;

static DBCLIENT: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub async fn connect_redis(uri: &str) -> Result<Client, Error> {
    let client = Client::open(uri).unwrap();
    Ok(client)
}

pub async fn get_database() -> &'static DatabaseConnection {
    DBCLIENT
        .get_or_init(|| async {
            let uri = conf::get_database_url();
            let mut opt = ConnectOptions::new(uri);
            opt.max_connections(100)
                .min_connections(5)
                .connect_timeout(Duration::from_secs(8))
                .idle_timeout(Duration::from_secs(8))
                .max_lifetime(Duration::from_secs(8))
                .sqlx_logging(false);
            Database::connect(opt).await.unwrap()
        })
        .await
}
