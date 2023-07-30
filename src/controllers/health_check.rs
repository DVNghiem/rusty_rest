// use crate::connect::mongodb::get_database;
use crate::orm::mongo_orm::MongoORM;
use actix_web::{HttpResponse, web};
use bson::{Bson, Document};
use mongodb::Client;
use serde::{Deserialize, Serialize};

const DB_NAME: &str = "rusty";
const COLL_NAME: &str = "users";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MyData {
    name: String,
    age: u8,
}

impl TryInto<Document> for MyData {
    type Error = bson::ser::Error;

    fn try_into(self) -> Result<Document, Self::Error> {
        let bson = bson::to_bson(&self)?;
        match bson {
            Bson::Document(doc) => Ok(doc),
            _ => Err(bson::ser::Error::InvalidCString("".to_string())),
        }
    }
}

pub async fn health_check(client: web::Data<Client>) -> HttpResponse {
    let db = client.database(DB_NAME);
    let test_col = MongoORM::new(db.collection("test"));

    let my_data = MyData {
        name: "John".to_string(),
        age: 30,
    };
    let res = test_col.insert_one(my_data.clone().try_into().unwrap()).await.unwrap();
    println!("{:?}", res);
    HttpResponse::Ok().json(my_data)
}
