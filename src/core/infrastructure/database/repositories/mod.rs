use super::connect_database;
use sea_orm::entity::prelude::*;

pub trait RepositoryTrait<T: EntityTrait> {
    type Model;
    fn new(table: String, schema: String) -> Self;
    fn find_all(&self)
        -> impl std::future::Future<Output = Result<Vec<Self::Model>, DbErr>> + Send;
}

#[derive(Clone)]
pub struct Repository<T: EntityTrait> {
    pub table: String,
    pub schema: String,
    pub model: T,
}

impl<T: EntityTrait> Default for Repository<T> {
    fn default() -> Self {
        let default_model = T::default();
        Self {
            table: default_model.table_name().to_string(),
            schema: default_model.schema_name().unwrap_or("public").to_string(),
            model: default_model,
        }
    }
}

impl<T: EntityTrait> RepositoryTrait<T> for Repository<T> {
    type Model = T::Model;

    fn new(table: String, schema: String) -> Self {
        let model = T::default();
        Self {
            table,
            schema,
            model,
        }
    }

    async fn find_all(&self) -> Result<Vec<Self::Model>, DbErr> {
        let conn = connect_database().await;
        let query = T::find().all(conn).await;
        query
    }
}
