use crate::connect::get_database;
use celery::prelude::*;
use entity::prelude::Post;
use sea_orm::EntityTrait;
use serde_json::{json, Value};

#[celery::task]
pub async fn add_post() -> TaskResult<Value> {
    let db = get_database().await;
    let r: Vec<Value> = Post::find().into_json().all(&db.clone()).await.unwrap();
    Ok(json!(r))
}
