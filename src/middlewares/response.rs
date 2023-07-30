use std::future::{ready, Ready};

use actix_web::{
    body::{BoxBody, EitherBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::StatusCode,
    web, Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, json, Value};

#[derive(Deserialize, Serialize)]
pub struct Response;

pub struct ResponseMiddleware<S> {
    service: S,
}

impl<S, B> Transform<S, ServiceRequest> for Response
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static + actix_web::body::MessageBody,
{
    type Response = ServiceResponse<EitherBody<BoxBody, B>>;
    type Error = Error;
    type InitError = ();
    type Transform = ResponseMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ResponseMiddleware { service }))
    }
}

impl<S, B> Service<ServiceRequest> for ResponseMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static + actix_web::body::MessageBody,
{
    type Response = ServiceResponse<EitherBody<BoxBody, B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let service: ServiceResponse<B> = fut.await?;
            let (_req, _res) = service.into_parts();
            let (part, body) = _res.into_parts();
            let bytes_body = match actix_web::body::to_bytes(body).await {
                Ok(data) => data,
                _ => web::Bytes::from(""),
            };
            let (_body, status): (Value, u16) = match from_slice(&bytes_body) {
                Ok(d) => (d, 200),
                Err(e) => (json!(e.to_string()), 500),
            };

            if status == 500 {
                return Ok(ServiceResponse::new(
                    _req,
                    HttpResponse::InternalServerError()
                        .body(
                            serde_json::to_string(&json!(
                            {
                                "data": "",
                                "msg": "",
                                "error": _body
                            }))
                            .unwrap(),
                        )
                        .map_into_left_body()
                ));
            }

            let status_code = part.status();

            let new_res = ServiceResponse::new(
                _req,
                match status_code {
                    StatusCode::OK => HttpResponse::Ok().body(
                        serde_json::to_string(&json!(
                        {
                            "data": _body,
                            "msg": "",
                            "error": ""
                        }))
                        .unwrap(),
                    ),
                    StatusCode::BAD_REQUEST => HttpResponse::BadRequest().body(
                        serde_json::to_string(&json!(
                        {
                            "data": "",
                            "msg": _body,
                            "error": "Bad request"
                        }))
                        .unwrap(),
                    ),
                    _ => HttpResponse::InternalServerError().body(
                        serde_json::to_string(&json!(
                        {
                            "data": "",
                            "msg": _body,
                            "error": "Internal server"
                        }))
                        .unwrap(),
                    ),
                }
                .map_into_left_body(),
            );
            Ok(new_res)
        })
    }
}
