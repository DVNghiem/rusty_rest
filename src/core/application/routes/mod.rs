use crate::core::application::controllers::health_check;
use actix_web::web;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "BasicAPI",
        version = "0.1.0",
    ),
    paths(
        health_check::health_check
    ),
    tags((name = "BasicAPI", description = "A very Basic API")),
)]
pub struct CoreApiDoc;

pub fn core_routing(cfg: &mut web::ServiceConfig, path: &str) {
    cfg.service(web::scope(path).route("/health_check", web::get().to(health_check::health_check)));
}
