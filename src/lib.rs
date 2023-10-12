use actix_web::{middleware, web, App, HttpServer};
use qdrant_client::qdrant::{CreateCollection, Distance, VectorParams, VectorsConfig};
use sqlx::postgres::PgPoolOptions;

use crate::operators::qdrant_operator::get_qdrant_connection;

pub mod data;
pub mod errors;
pub mod handlers;
pub mod operators;

pub fn check_environment_variables() -> Result<(), String> {
    std::env::var("API_KEY").map_err(|_| "API_KEY environment variable not set.")?;
    std::env::var("EMBEDDING_SERVER_CALL")
        .map_err(|_| "EMBEDDING_SERVER_CALL environment variable not set.")?;
    std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL environment variable not set.")?;
    Ok(())
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let env_result = check_environment_variables();
    if let Err(e) = env_result {
        panic!("{}", e);
    }

    let database_url = std::env::var("DATABASE_URL")
        .expect("POSTGRES_CONNECTION_URL environment variable not set.");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate database.");

    let qdrant_client = get_qdrant_connection().await.unwrap();
    let qdrant_collection =
        std::env::var("QDRANT_COLLECTION").unwrap_or("doc_embeddings".to_owned());
    let embedding_size = std::env::var("EMBEDDING_SIZE").unwrap_or("1024".to_owned());
    let embedding_size = embedding_size.parse::<u64>().unwrap_or(1024);

    let _ = qdrant_client
        .create_collection(&CreateCollection {
            collection_name: qdrant_collection,
            vectors_config: Some(VectorsConfig {
                config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                    VectorParams {
                        size: embedding_size,
                        distance: Distance::Cosine.into(),
                        hnsw_config: None,
                        quantization_config: None,
                        on_disk: None,
                    },
                )),
            }),
            ..Default::default()
        })
        .await
        .map_err(|err| {
            log::info!("Failed to create collection: {:?}", err);
        });

    log::info!("starting HTTP server at http://localhost:8090");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api")
                    .route(
                        "/healthcheck",
                        web::get().to(handlers::healthcheck),
                    )
                    .route(
                        "/check_key",
                        web::get().to(handlers::auth_handler::check_key),
                    )
                    .route(
                        "/index_document",
                        web::post().to(handlers::embedding_handler::index_document),
                    )
                    .service(
                        web::resource("/document_group")
                            .route(
                                web::post().to(handlers::doc_group_handler::create_document_group),
                            )
                            .route(
                                web::put().to(handlers::doc_group_handler::index_document_group),
                            ),
                    )
                    .route(
                        "/search",
                        web::post().to(handlers::search_handler::semantic_search),
                    ),
            )
    })
    .bind(("0.0.0.0", 8090))?
    .run()
    .await
}
