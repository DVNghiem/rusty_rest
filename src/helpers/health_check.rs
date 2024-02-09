use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter};
use serde_json::Value;
use entity::prelude::Post;
use crate::errors::Error;


pub struct HealthCheckHelper;

impl HealthCheckHelper {
    pub async fn find_all(&self, db: &DatabaseConnection) -> Result<Vec<Value>, Error> {
        let data = Post::find().into_json().all(db).await;
        match data {
            Ok(model) => {
                Ok(model)
            },
            Err(_) => Err(Error::DbError(String::from("Find error")))
        }

    }
}