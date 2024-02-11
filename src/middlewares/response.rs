use actix_web::{
    body::{BoxBody, EitherBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::{header, StatusCode},
    web, Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, json, Value};
use std::{
    future::{ready, Ready},
    str::from_utf8,
};
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

            let new_res = ServiceResponse::new(
                _req.clone(),
                match _res.status() {
                    StatusCode::OK => {
                        let header_map = _res.headers();
                        match header_map.get(header::CONTENT_TYPE) {
                            Some(t) => {
                                let header_value_str = t.to_str().unwrap().to_lowercase();
                                if header_value_str == String::from("application/json") {
                                    let body = _res.into_body();
                                    let bytes_body = match actix_web::body::to_bytes(body).await {
                                        Ok(data) => data,
                                        _ => web::Bytes::from(""),
                                    };
                                    let (_body, status): (Value, u16) =
                                        match from_slice(&bytes_body) {
                                            Ok(d) => (d, 200),
                                            Err(e) => (json!(e.to_string()), 500),
                                        };
                                    if status == 500 {
                                        return Ok(ServiceResponse::new(
                                            _req,
                                            HttpResponse::InternalServerError()
                                                .json(json!({
                                                    "data": "",
                                                    "error_msg": _body,
                                                    "error_code": "INTERNAL_SERVER_ERR"
                                                }))
                                                .map_into_left_body(),
                                        ));
                                    }
                                    HttpResponse::Ok().json(json!(
                                    {
                                        "data": _body,
                                        "error_msg": "",
                                        "error_code": ""
                                    }))
                                } else if header_value_str == String::from("text/plain") {
                                    let body = _res.into_body();
                                    let bytes_body = match actix_web::body::to_bytes(body).await {
                                        Ok(data) => data,
                                        _ => web::Bytes::from(""),
                                    };
                                    HttpResponse::Ok().json(json!(
                                    {
                                        "data": from_utf8(&bytes_body).unwrap(),
                                        "error_msg": "",
                                        "error_code": ""
                                    }))
                                } else {
                                    _res.map_into_boxed_body()
                                }
                            }
                            _ => _res.map_into_boxed_body(),
                        }
                    }
                    _ => _res.map_into_boxed_body(),
                }
                .map_into_left_body(),
            );
            Ok(new_res)
        })
    }
}
