#![allow(dead_code)]

use rusty_rest::middlewares;
// use crate::config::conf;
use rusty_rest::{connect, routes::routing};
use rusty_rest::tasks::health_check::add_post;
use actix_web::{middleware::Logger, web, App, HttpServer};
use env_logger::Env;
use rusty_rest::config::Config;
use std::net::Ipv4Addr;
use structopt::StructOpt;
use rusty_rest::worker::create_worker;

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
    let opt = RunOpt::from_args();
    let worker = create_worker().await;

    match opt {
        RunOpt::Web { num_worker } => {
            worker.send_task(add_post::new()).await.unwrap();
            let redis_db = connect::connect_redis()
                .await;
            connect::connect_database().await;
            HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(redis_db.clone()))
                    .app_data(web::Data::new(worker.clone()))
                    .configure(routing)
                    // .wrap(NormalizePath::trim())
                    .wrap(middlewares::response::Response)
                    .wrap(actix_web::middleware::Compress::default())
                    .wrap(Logger::new("%a %r %s [%b bytes] %T seconds"))
            })
            .workers(num_worker)
            .bind((Ipv4Addr::UNSPECIFIED, conf.port))?
            .run()
            .await
        }
        RunOpt::Worker { queues } => {
            worker.display_pretty().await;
            let queue_vec: Vec<&str> = queues.split(",").map(|x| x.trim()).collect();
            worker.consume_from(&queue_vec).await.unwrap();
            worker.close().await.unwrap();
            Ok(())
        }
    }
}
