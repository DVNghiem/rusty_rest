use lazy_static::lazy_static;
use redis::Client;
use std::sync::Mutex;

lazy_static! {
    pub static ref REDIS_CON: Mutex<Client> = {
        let client = Client::open("").unwrap();
        Mutex::new(client)
    };
}
