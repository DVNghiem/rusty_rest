use actix_web::web;

use super::controllers::post;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "BasicAPI",
        version = "0.1.0",
    ),
    paths(
    ),
    tags((name = "BasicAPI", description = "A very Basic API")),
)]
pub struct ApiDoc;

pub fn api_routing(cfg: &mut web::ServiceConfig, path: &str) {
    cfg.service(web::scope(path).route("/get_post", web::get().to(post::get_post)));
}
