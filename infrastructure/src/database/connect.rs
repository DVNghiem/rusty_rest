use crate::config::Config;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;
use tokio::sync::OnceCell;

pub enum DatabaseType {
    Writer,
    Reader,
}

static DB_WRITER: OnceCell<DatabaseConnection> = OnceCell::const_new();
static DB_READER: OnceCell<DatabaseConnection> = OnceCell::const_new();

async fn create_connection() -> DatabaseConnection {
    let url = Config::new().database_url;
    let mut opt = ConnectOptions::new(url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(false);
    Database::connect(opt).await.unwrap()
}

pub async fn connect_database(database_type: DatabaseType) -> &'static DatabaseConnection {
    match database_type {
        DatabaseType::Writer => {
            DB_WRITER
                .get_or_init(|| async { create_connection().await })
                .await
        }
        DatabaseType::Reader => {
            DB_READER
                .get_or_init(|| async { create_connection().await })
                .await
        }
    }
}
