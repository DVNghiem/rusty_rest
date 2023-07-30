#![allow(dead_code)]

// define module
mod config;
mod connect;
mod controllers;
mod helpers;
mod middlewares;
mod routes;
mod errors;

use  crate::config::conf;
use crate::routes::routing;
use actix_web::{
    middleware::{DefaultHeaders, Logger, NormalizePath},
    App, HttpServer, web,
};

use env_logger::Env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    conf::init();

    // mongodb
    let client_db = connect::mongodb::get_client(&conf::get_database_url()).await.unwrap();
    // redis
    let redis_db = connect::redis::redis_client(&conf::get_redis_url()).await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client_db.clone()))
            .app_data(web::Data::new(redis_db.clone()))
            .configure(routing)
            .wrap(NormalizePath::trim())
            .wrap(middlewares::response::Response)
            .wrap(DefaultHeaders::new().add(("Content-type", "application/json")))
            .wrap(Logger::new("%a %{User-Agent}i"))
    })
    .workers(1)
    .bind((conf::get_host(), conf::get_port()))?
    .run()
    .await
}


