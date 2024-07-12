use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate, Clone, IntoParams, ToSchema)]
pub struct GetPostDto {
    #[validate(length(min = 1))]
    title: String,
}
