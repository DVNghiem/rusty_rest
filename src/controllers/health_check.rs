use actix_web::{HttpResponse, web};
use mongodb::Client;
use serde::{Deserialize, Serialize};

const DB_NAME: &str = "rusty";
const COLL_NAME: &str = "users";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MyData {
    name: String,
    age: u8,
}

pub async fn health_check(client: web::Data<Client>) -> HttpResponse {
    let db = client.database(DB_NAME);
    let test_col: mongodb::Collection<MyData> =db.collection("test");

    let my_data = MyData {
        name: "John".to_string(),
        age: 30,
    };
    let res = test_col.insert_one(my_data.clone(), None).await.unwrap();
    println!("{:?}", res);
    HttpResponse::Ok().json(my_data)
}
