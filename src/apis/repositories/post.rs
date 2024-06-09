use crate::core::infrastructure::database::repositories::{Repository, RepositoryTrait};
use entity::{post::Model, prelude::Post};

pub trait PostRepositoryTrait {
    fn find_all(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<Model>, sea_orm::DbErr>> + Send;
}

#[derive(Clone)]
pub struct PostRepository {
    pub repository: Repository<Post>,
}

impl Default for PostRepository {
    fn default() -> Self {
        Self {
            repository: Repository::default(),
        }
    }
}

impl PostRepositoryTrait for PostRepository {
    async fn find_all(&self) -> Result<Vec<Model>, sea_orm::DbErr> {
        let query = self.repository.find_all().await;
        query
    }
}
