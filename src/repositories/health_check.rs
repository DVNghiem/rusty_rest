use crate::connect::connect_database;
use entity::prelude::Post;
use sea_orm::EntityTrait;
use serde_json::Value;

use crate::errors::Error;

pub struct HealthCheckRepository;

impl HealthCheckRepository {

    pub async fn find_all(&self) -> Result<Vec<Value>, Error> {
        let db = connect_database().await;
        let data = Post::find().into_json().all(db).await;
        match data {
            Ok(model) => Ok(model),
            Err(_) => Err(Error::DbError(String::from("Find error"))),
        }
    }
}
