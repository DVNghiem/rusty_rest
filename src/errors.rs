use actix_web::{error, http::StatusCode, HttpResponse};
use serde_json::{from_str, json, Value};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("direct redis error: {0}")]
    RedisError(#[from] RedisErr),

    #[error("{0}")]
    DbError(String)
}

#[derive(Error, Debug)]
pub enum RedisErr {
    #[error("Redis key error")]
    RedisKeyErr,
    #[error("Redis error : {0}")]
    RedisErr(redis::RedisError),
}

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("{0}")]
    InputInvalid(Value),

    #[error("Login fail")]
    LoginFail,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Access token incorrect")]
    AccessTokenIncorrect,

    #[error("Access token is expired")]
    AccessTokenExpire,
}

impl error::ResponseError for HttpError {
    fn error_response(&self) -> HttpResponse {
        let error_full_str = format!("{:?}", self);
        let error_code: Vec<&str> = error_full_str.split('(').collect();
        let error_msg_str = self.to_string();
        let error_msg = from_str::<Value>(error_msg_str.as_str()).unwrap();
        let data = json!(
            {
                "data": "",
                "error_code": error_code[0],
                "error_msg": error_msg
            }
        );
        HttpResponse::build(self.status_code()).json(data)
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            HttpError::InputInvalid(_) => StatusCode::BAD_REQUEST,
            HttpError::LoginFail => StatusCode::UNAUTHORIZED,
            HttpError::AccessTokenExpire => StatusCode::FORBIDDEN,
            HttpError::AccessTokenIncorrect => StatusCode::FORBIDDEN,
            HttpError::PermissionDenied => StatusCode::FORBIDDEN,
            // _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
