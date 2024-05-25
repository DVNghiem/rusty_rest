use crate::{errors::Error, repositories::health_check::HealthCheckRepository};
use entity::prelude::Post;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde_json::Value;

pub struct HealthCheckHelper {
    repository: HealthCheckRepository,
}

impl HealthCheckHelper {
    pub fn new() -> Self {
        HealthCheckHelper {
            repository: HealthCheckRepository,
        }
    }

    pub async fn find_all(&self, db: &DatabaseConnection) -> Result<Vec<Value>, Error> {
        let data = Post::find().into_json().all(db).await;
        match data {
            Ok(model) => Ok(model),
            Err(_) => Err(Error::DbError(String::from("Find error"))),
        }
    }
}
