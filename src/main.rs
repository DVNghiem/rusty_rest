#![allow(dead_code)]
// define module
mod config;
mod connect;
mod controllers;
mod errors;
mod helpers;
mod middlewares;
mod routes;
mod schema;
mod tasks;
mod worker;

use std::net::Ipv4Addr;
use crate::config::conf;
use crate::routes::routing;
use actix_web::{
    middleware::Logger,
    web, App, HttpServer,
};
use env_logger::Env;
use structopt::StructOpt;
use worker::create_worker;
// use crate::tasks::health_check::add_post;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "rusty_app",
    about = "Run a Rust Worker or Backend web.",
    setting = structopt::clap::AppSettings::ColoredHelp,
)]
enum RunOpt {
    Worker {
        #[structopt(short = "-q", default_value = "test_queue", about="Queue name to consume from split by comma. e.g: test_queue, test_queue2, test_queue3")]
        queues: String,
    
    },
    Web {
        #[structopt(short = "w", default_value = "1", about="Number of worker to run.")]
        num_worker: usize,
    },
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    conf::init();
    let opt = RunOpt::from_args();
    let worker = create_worker().await;

    match opt {
        RunOpt::Web { num_worker } => {
            // worker.send_task(add_post::new()).await.unwrap();
            let redis_db = connect::connect_redis(&conf::get_redis_url())
                .await
                .unwrap();
            let db = connect::get_database().await;
            HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(db.clone()))
                    .app_data(web::Data::new(redis_db.clone()))
                    .app_data(web::Data::new(worker.clone()))
                    .configure(routing)
                    // .wrap(NormalizePath::trim())
                    .wrap(middlewares::response::Response)
                    .wrap(actix_web::middleware::Compress::default())
                    .wrap(Logger::new("%a %r %s [%b bytes] %T seconds"))
            })
            .workers(num_worker)
            .bind((Ipv4Addr::UNSPECIFIED, conf::get_port()))?
            .run()
            .await
        }
        RunOpt::Worker{queues} => {
            worker.display_pretty().await;
            let queue_vec: Vec<&str> = queues.split(",").map(|x| x.trim()).collect();
            worker.consume_from(&queue_vec).await.unwrap();
            worker.close().await.unwrap();
            Ok(())
        }
    }
}
