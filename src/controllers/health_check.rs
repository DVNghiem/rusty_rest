use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde_json::json;
use validator::Validate;
use crate::errors::HttpError;
use crate::schema::health_check::HealthCheckSchema;
use crate::helpers::health_check::HealthCheckHelper;

const HELPER: HealthCheckHelper = HealthCheckHelper;

pub async fn health_check(data: web::Query<HealthCheckSchema>, db: web::Data<DatabaseConnection>) -> Result<HttpResponse, HttpError> {

    match data.validate() {
        Ok(_) => {
            let result = HELPER.find_all(db.as_ref()).await;
            match result {
                Ok(r) => Ok(HttpResponse::Ok().json(r)),
                Err(_) => Err(HttpError::LoginFail)
            }
        },
        Err(e) => Err(HttpError::InputInvalid(json!(e)))
    }
}