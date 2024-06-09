use actix_web::web;

use super::controllers::post;

pub fn api_routing(cfg: &mut web::ServiceConfig, path: &str) {
   
    cfg.service(
        web::scope(path)
            .route("/get_post", web::get().to(post::get_post)),
    );
}