#![allow(dead_code)]

use actix_web::{middleware::Logger, App, HttpServer};
use env_logger::Env;
use rusty_rest::{config::Config, router::router};
use std::net::Ipv4Addr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "rusty_app",
    about = "Run a Rust Worker or Backend web.",
    setting = structopt::clap::AppSettings::ColoredHelp,
)]
enum RunOpt {
    Worker {
        #[structopt(
            short = "-q",
            default_value = "test_queue",
            about = "Queue name to consume from split by comma. e.g: test_queue, test_queue2, test_queue3"
        )]
        queues: String,
    },
    Web {
        #[structopt(short = "w", default_value = "1", about = "Number of worker to run.")]
        num_worker: usize,
    },
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let conf = Config::new();
    
    HttpServer::new(move || {
        App::new()
            .configure(router)
            .wrap(actix_web::middleware::Compress::default())
            .wrap(Logger::new("%a %r %s [%b bytes] %T seconds"))
    })
    .workers(1)
    .bind((Ipv4Addr::UNSPECIFIED, conf.port))?
    .run()
    .await
}
