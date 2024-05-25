use crate::{errors::Error, repositories::health_check::HealthCheckRepository};
use serde_json::Value;

pub struct HealthCheckHelper {
    repository: HealthCheckRepository,
}

impl HealthCheckHelper {
    
    pub fn new() -> Self {
        HealthCheckHelper {
            repository: HealthCheckRepository,
        }
    }

    pub async fn find_all(&self) -> Result<Vec<Value>, Error> {
        self.repository.find_all().await
    }
}
