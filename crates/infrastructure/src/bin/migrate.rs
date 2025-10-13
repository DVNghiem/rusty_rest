use sqlx::{Connection, PgConnection};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    println!("Connecting to database...");
    let mut conn = PgConnection::connect(&database_url).await?;
    
    println!("Running migrations...");
    let migrator = sqlx::migrate!("./migrations");
    migrator.run(&mut conn).await?;
    
    println!("✅ Migrations completed successfully!");
    
    Ok(())
}