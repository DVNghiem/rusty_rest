use redis::Client;
use crate::errors::Error;

pub async fn redis_client(uri: &str) -> Result<Client, Error>{
    let client = Client::open(uri).unwrap();
    Ok(client)
}