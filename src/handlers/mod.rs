use actix_web::{Responder, HttpResponse};
pub mod auth_handler;
pub mod doc_group_handler;
pub mod embedding_handler;
pub mod search_handler;

pub async fn healthcheck() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
