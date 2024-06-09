use crate::core::application::routes::core_routing;
use crate::apis::routes::api_routing;
use actix_web::web;

pub fn router(cfg: &mut web::ServiceConfig) {
    core_routing(cfg, "core");
    api_routing(cfg, "");
}
