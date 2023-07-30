use actix_web::{HttpResponse, web};
use mongodb;
use redis::{self, AsyncCommands, aio::Connection};
use serde::{Deserialize, Serialize};

const DB_NAME: &str = "rusty";
const COLL_NAME: &str = "users";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MyData {
    name: String,
    age: u8,
}

pub async fn health_check(client: web::Data<mongodb::Client>, rdb: web::Data<redis::Client>) -> HttpResponse {
    let db = client.database(DB_NAME);
    let test_col: mongodb::Collection<MyData> =db.collection("test");

    let my_data = MyData {
        name: "John".to_string(),
        age: 30,
    };
    let res = test_col.insert_one(my_data.clone(), None).await.unwrap();
    println!("{:?}", res);

    let mut rdb_conn : Connection = rdb.get_async_connection().await.unwrap();
    let _ : () = rdb_conn.set("test", "123").await.unwrap();
    HttpResponse::Ok().json(my_data)
}
