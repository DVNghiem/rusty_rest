use crate::{
    apis::{
        dtos::post::GetPostDto,
        repositories::post::{PostRepository, PostRepositoryTrait},
    },
    core::application::handlers::RequestHandler,
};
use actix_web::web;
use entity::post::Model;
use validator::{Validate, ValidationErrors};

#[derive(Default)]
pub struct GetPostHandler;

impl RequestHandler for GetPostHandler {
    type Input = web::Query<GetPostDto>;
    type Output = Vec<Model>;
    type Validation = GetPostDto;

    fn validate(&self, value: Self::Validation) -> Result<(), ValidationErrors> {
        return value.validate();
    }

    async fn handler(&self, request: Self::Input) -> Result<Self::Output, String> {
        
        match self.validate(request.into_inner()) {
            Ok(_) => {
                let repository = PostRepository::default();
                let result = repository.find_all().await;
                match result {
                    Ok(data) => Ok(data),
                    Err(err) => Err(err.to_string()),
                }
            }
            Err(err) => return Err(err.to_string()),
        }
    }
}
