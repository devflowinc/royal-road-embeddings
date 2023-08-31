use royal_road_embeddings::{
    errors::ErrorResponse,
    handlers::embedding_handler::{IndexDocumentRequest, IndexDocumentResponse},
};

use either::Either;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(transparent)]
struct IndexDocumentReturn {
    #[serde(with = "either::serde_untagged")]
    response: Either<IndexDocumentResponse, ErrorResponse>,
}
#[actix_rt::test]
async fn test_index_document() {
    let key = "key";
    let req = reqwest::Client::new();
    let document = IndexDocumentRequest {
        doc_html: "html".to_string(),
        story_id: 5,
        index: 5,
    };

    let response = req
        .post("http://localhost:8090/api/index_document")
        .header("X-API-KEY", key)
        .json(&document)
        .send()
        .await;
    assert!(response.is_ok());
    let res = response.unwrap();
    let json = res.json::<IndexDocumentReturn>().await.unwrap();
    match json.response {
        Either::Left(_a) => (),
        Either::Right(b) => {
            panic!("{:?} code {:}", b.message, b.error_code);
        }
    }
}

#[actix_rt::test]
async fn check_auth() {
    let key = "key";

    let res = reqwest::Client::new()
        .get("http://localhost:8090/api/check_key")
        .header("X-API-KEY", key)
        .send()
        .await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap().status(), 204);
}
