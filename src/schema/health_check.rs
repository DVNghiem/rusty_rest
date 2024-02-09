use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct HealthCheckSchema {
    #[validate(range(min = 18, max = 20))]
    age: u32,
}
