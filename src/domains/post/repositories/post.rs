use entity::{post::Model, prelude::Post};
use infrastructure::database::connect::{connect_database, DatabaseType};
use sea_orm::EntityTrait;

pub trait PostRepositoryTrait {
    fn find_all(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<Model>, sea_orm::DbErr>> + Send;
}

#[derive(Clone)]
pub struct PostRepository;

impl Default for PostRepository {
    fn default() -> Self {
        Self {}
    }
}

impl PostRepositoryTrait for PostRepository {
    async fn find_all(&self) -> Result<Vec<Model>, sea_orm::DbErr> {
        let conn = connect_database(DatabaseType::Reader).await;
        let posts = Post::find().all(conn).await?;
        Ok(posts)
    }
}
