
use actix_web;
use rusty_rest::application::app::Application;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let application = Application::new();
    application.run().await
}
