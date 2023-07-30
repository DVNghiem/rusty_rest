use crate::errors::Error;
use mongodb::Client;

pub async fn get_client() -> Result<Client, Error> {
    let client = Client::with_uri_str("mongodb://localhost:27017/")
        .await
        .unwrap();
    Ok(client)
}
