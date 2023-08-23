use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;

mod data;
mod errors;
mod handlers;
mod operators;

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres.");
    let migration_result = sqlx::migrate!()
        .run(&pool)
        .await;

    match migration_result {
        Ok(_) => log::info!("Successfully migrated database."),
        Err(e) => log::error!("Failed to migrate database: {}", e),
    }

    log::info!("starting HTTP server at http://localhost:8090");

    HttpServer::new(move || {
        App::new().app_data(web::Data::new(pool.clone())).service(
            web::scope("/api")
        )
    })
    .bind(("0.0.0.0", 8090))?
    .run()
    .await
}
