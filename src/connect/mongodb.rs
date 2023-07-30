use crate::errors::Error;
use mongodb::Client;


pub async fn get_client(uri: &str) -> Result<Client, Error> {
    let client = Client::with_uri_str(uri)
        .await
        .unwrap();
    Ok(client)
}
