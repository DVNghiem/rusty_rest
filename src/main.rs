#![allow(dead_code)]

// define module
mod config;
mod connect;
mod controllers;
mod helpers;
mod middlewares;
mod models;
mod orm;
mod routes;
mod errors;


use crate::routes::routing;
use actix_web::{
    middleware::{DefaultHeaders, Logger, NormalizePath},
    App, HttpServer, web,
};

use dotenv::dotenv;
use env_logger::Env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let conf = config::Config::from_env();

    // mongodb
    let client_db = connect::mongodb::get_client().await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client_db.clone()))
            .configure(routing)
            .wrap(NormalizePath::trim())
            .wrap(middlewares::response::Response)
            .wrap(DefaultHeaders::new().add(("Content-type", "application/json")))
            .wrap(Logger::new("%a %{User-Agent}i"))
    })
    .workers(1)
    .bind((conf.get_host().to_owned(), conf.get_port()))?
    .run()
    .await
}


