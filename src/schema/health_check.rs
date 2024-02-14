use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate, Clone, IntoParams, ToSchema)]
pub struct HealthCheckSchema {
    #[validate(range(min = 18, max = 20))]
    age: u32,
}
