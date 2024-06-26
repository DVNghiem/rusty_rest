pub mod repositories;

use crate::config::Config;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;
use tokio::sync::OnceCell;

pub static DB: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub async fn connect_database() -> &'static DatabaseConnection {
    let url = Config::new().database_url;
    DB.get_or_init(|| async {
        let mut opt = ConnectOptions::new(url);
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
