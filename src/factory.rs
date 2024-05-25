use crate::helpers::health_check::HealthCheckHelper;

pub struct Factory;

impl Factory {

    pub fn new() -> Self {
        Factory {}
    }

    /// The function `get_health_check_helper` returns a new instance of `HealthCheckHelper`.
    /// 
    /// Returns:
    /// 
    /// An instance of the `HealthCheckHelper` struct is being returned.
    pub fn get_health_check_helper(&self) -> HealthCheckHelper {
        HealthCheckHelper::new()
    }

}


