use crate::errors::Error;
use entity::prelude::Post;
use sea_orm::{DatabaseConnection, EntityOrSelect, EntityTrait, QueryFilter, QuerySelect};
use serde_json::Value;

pub struct HealthCheckHelper;

impl HealthCheckHelper {
    pub async fn find_all(&self, db: &DatabaseConnection) -> Result<Vec<Value>, Error> {
        let data = Post::find().into_json().all(db).await;
        match data {
            Ok(model) => Ok(model),
            Err(_) => Err(Error::DbError(String::from("Find error"))),
        }
    }
}
