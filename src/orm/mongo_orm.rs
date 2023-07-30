use std::borrow::Borrow;

use crate::errors::Error;
use bson::{self, oid::ObjectId, Document};
use futures_util::TryStreamExt;
use mongodb::{
    options::{FindOneOptions, FindOptions},
    Collection
};

pub struct MongoORM {
    collection: Collection<Document>,
}

impl MongoORM {

    pub fn new(collection: Collection<Document>) -> Self{
        Self { collection }
    }

    pub async fn insert_one(&self, data: Document) -> Result<ObjectId, Error> {
        let bson = bson::to_bson(&data).unwrap();
        let doc = bson.as_document().unwrap().borrow();
        let result = self.collection.insert_one(doc, None).await.unwrap();
        Ok(result.inserted_id.as_object_id().unwrap().clone())
    }

    pub async fn find_one(
        &self,
        filter: Document,
        projection: Option<Document>,
    ) -> Result<Option<Document>, Error> {
        let options = match projection {
            Some(p) => FindOneOptions::builder().projection(p).build(),
            _ => FindOneOptions::builder().build(),
        };
        let result = self.collection.find_one(filter, options).await.unwrap();
        Ok(result)
    }

    pub async fn find(&self, filter: Document, projection: Option<Document>)-> Result<Vec<Document>, Error>{

        let options = match projection {
            Some(p) => FindOptions::builder().projection(p).build(),
            _ => FindOptions::builder().build(),
        };
        let mut cursor = self.collection.find(filter, options).await.unwrap();
        let mut data : Vec<Document> = Vec::new();
        
        while let Some(item) = cursor.try_next().await.unwrap() {
            data.push(item)
        }
        Ok(data)
    }
}
