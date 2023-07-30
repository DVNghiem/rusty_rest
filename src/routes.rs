
use actix_web::web;
use crate::controllers::health_check;


pub fn routing(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .route("/health_check", web::get().to(health_check::health_check))
    );
}