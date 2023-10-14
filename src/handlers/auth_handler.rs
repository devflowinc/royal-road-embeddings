use std::future::{ready, Ready};

use actix_web::{
    HttpRequest, HttpResponse,
    {dev::Payload, FromRequest},
};

use crate::errors::ServiceError;

pub struct AuthRequired {}

impl FromRequest for AuthRequired {
    type Error = actix_web::Error;
    type Future = Ready<Result<AuthRequired, actix_web::Error>>;

    fn from_request(req: &HttpRequest, _pl: &mut Payload) -> Self::Future {
        let key = std::env::var("API_KEY").expect("API_KEY must be set");
        if let Some(api_key) = req.headers().get("Authorization") {
            if api_key.to_str().unwrap() == key {
                return ready(Ok(AuthRequired {}));
            }
        }

        ready(Err(ServiceError::InvalidAPIKey.into()))
    }
}

pub async fn check_key(_: AuthRequired) -> Result<HttpResponse, ServiceError> {
    Ok(HttpResponse::NoContent().finish())
}
