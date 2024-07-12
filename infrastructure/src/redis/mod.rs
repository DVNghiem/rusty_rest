use crate::config::Config;
use redis::Client;
use tokio::sync::OnceCell;

pub static REDIS: OnceCell<Client> = OnceCell::const_new();

pub async fn connect_redis() -> Client {
    let uri = Config::new().redis_url;
    REDIS
        .get_or_init(|| {
            Box::pin(async {
                let client = Client::open(uri).unwrap();
                client
            })
        })
        .await
        .clone()
}
