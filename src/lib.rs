use actix_web::{web, App, HttpServer};

mod data;
mod errors;
mod handlers;
mod operators;

pub fn check_environment_variables() -> Result<(), String> {
    std::env::var("API_KEY").map_err(|_| "API_KEY environment variable not set.")?;
    std::env::var("EMBEDDING_SERVER_CALL")
        .map_err(|_| "EMBEDDING_SERVER_CALL environment variable not set.")?;
    Ok(())
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let env_result = check_environment_variables();
    if let Err(e) = env_result {
        panic!("{}", e);
    }

    log::info!("starting HTTP server at http://localhost:8090");

    HttpServer::new(move || {
        App::new()
            // .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api")
                    .route(
                        "/check_key",
                        web::get().to(handlers::auth_handler::check_key),
                    )
                    .route(
                        "/index_document",
                        web::post().to(handlers::embedding_handler::index_document),
                    ),
            )
    })
    .bind(("0.0.0.0", 8090))?
    .run()
    .await
}
