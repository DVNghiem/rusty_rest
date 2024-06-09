use actix_web::HttpResponse;
use serde_json::json;

#[utoipa::path(
    get,
    path = "/core/health_check",
    responses(
        (status = 200, description = "List current todo items")
    ),
    tag = "BasicAPI",
)]
pub async fn health_check() -> impl actix_web::Responder {
    HttpResponse::Ok().json(json!({
        "status": "ok"
    }))
}
