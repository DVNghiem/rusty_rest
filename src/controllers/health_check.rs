use crate::helpers::health_check::HealthCheckHelper;
use crate::schema::health_check::HealthCheckSchema;
use crate::{errors::HttpError, factory::Factory};
use actix_web::{web, HttpResponse};
use lazy_static::lazy_static;
use sea_orm::DatabaseConnection;
use serde_json::json;
use validator::Validate;

lazy_static! {
    static ref FACTORY: Factory = Factory::new();
    static ref HELPER: HealthCheckHelper = FACTORY.get_health_check_helper();
}

#[utoipa::path(
    get,
    path = "/health_check",
    params(HealthCheckSchema),
    responses(
        (status = 200, description = "List current todo items")
    ),
    tag = "BasicAPI",
)]
pub async fn health_check(
    data: web::Query<HealthCheckSchema>,
    db: web::Data<DatabaseConnection>,
) -> impl actix_web::Responder {
    match data.validate() {
        Ok(_) => {
            let result = HELPER.find_all(db.as_ref()).await;
            match result {
                Ok(r) => Ok(HttpResponse::Ok().json(r)),
                Err(_) => Err(HttpError::LoginFail),
            }
        }
        Err(e) => Err(HttpError::InputInvalid(json!(e))),
    }
}
