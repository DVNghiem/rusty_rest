use super::super::handlers::post::GetPostHandler;
use crate::domains::post::dtos::GetPostDto;
use actix_web::web;
use infrastructure::handler::RequestHandler;

pub async fn get_post(request: web::Query<GetPostDto>) -> impl actix_web::Responder {
    let handler = GetPostHandler::default();
    let result = handler.handler(request).await;
    match result {
        Ok(data) => actix_web::HttpResponse::Ok().json(data),
        Err(err) => actix_web::HttpResponse::InternalServerError().body(err),
    }
}
