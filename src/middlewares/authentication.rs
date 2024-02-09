use std::future::{ready, Ready};

use actix_web::{
    body::{BoxBody, EitherBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::LocalBoxFuture;

use crate::{config::conf, helpers::user::user_helper};

pub struct Authentication {
    pub role: Vec<String>,
}

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<BoxBody, B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware {
            service,
            role: self.role.clone(),
        }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: S,
    role: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<BoxBody, B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        match req.headers().get("Authorization") {
            Some(authen_header) => {
                let authen_str = authen_header.to_str().unwrap();
                if authen_str.starts_with("bearer") || authen_str.starts_with("Bearer") {
                    let token = authen_str[6..authen_str.len()].trim();
                    match user_helper::validate_jwt(token.to_owned(), &conf::get_access_key()) {
                        Ok(payload) => {
                            if !self.role.contains(&"ALL".to_owned())
                                && payload.role != "ADMIN".to_owned()
                            {
                                if !self.role.contains(&payload.role) {
                                    let (request, _pl) = req.into_parts();
                                    let response = HttpResponse::Forbidden()
                                        .json("PERMISION_DENY")
                                        .map_into_left_body();
                                    return Box::pin(async {
                                        Ok(ServiceResponse::new(request, response))
                                    });
                                }
                            }

                            req.extensions_mut().insert(payload);
                            let fut = self.service.call(req);
                            Box::pin(
                                async move { fut.await.map(ServiceResponse::map_into_right_body) },
                            )
                        }
                        Err(_err) => {
                            let err_str = _err.to_string();
                            let (request, _pl) = req.into_parts();
                            let response = HttpResponse::Unauthorized()
                                .json(err_str)
                                .map_into_left_body();
                            return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
                        }
                    }
                } else {
                    let (request, _pl) = req.into_parts();
                    let response = HttpResponse::Unauthorized()
                        .body("Type token not provide")
                        .map_into_left_body();
                    Box::pin(async { Ok(ServiceResponse::new(request, response)) })
                }
            }
            _ => {
                let (request, _pl) = req.into_parts();
                let response = HttpResponse::Unauthorized()
                    .json("User Invalid")
                    .map_into_left_body();
                return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
            }
        }
    }
}
