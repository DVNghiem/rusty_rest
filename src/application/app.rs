use std::net::Ipv4Addr;

use actix_web::{middleware::Logger, App, HttpServer};
use env_logger::Env;

use crate::config;

use super::router::router;

pub struct Application {
    pub config: config::Config,
}

impl Application {
    pub fn new() -> Self {
        let config = config::Config::new();
        Self { config }
    }

    pub async fn run(&self) -> std::io::Result<()> {
        env_logger::init_from_env(Env::default().default_filter_or("info"));

        HttpServer::new(move || {
            App::new()
                .configure(router)
                .wrap(actix_web::middleware::Compress::default())
                .wrap(Logger::new("%a %r %s [%b bytes] %T seconds"))
        })
        .workers(1)
        .bind((Ipv4Addr::UNSPECIFIED, self.config.port))?
        .run()
        .await
    }
}
