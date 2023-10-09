use actix_rt;
use royal_road_embeddings::{
    errors::ErrorResponse,
    handlers::{
        doc_group_handler::{GroupDocumentRequest, IndexDocumentGroupRequest},
        embedding_handler::{IndexDocumentRequest, IndexDocumentResponse},
    },
};

use either::Either;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(transparent)]
struct IndexDocumentReturn {
    #[serde(with = "either::serde_untagged")]
    response: Either<IndexDocumentResponse, ErrorResponse>,
}

async fn add_document(content: String, story_id: i64, index: i32) {
    let key = "key";
    let req = reqwest::Client::new();
    let document = IndexDocumentRequest {
        doc_html: content,
        story_id,
        index,
    };

    let response = req
        .post("http://localhost:8090/api/index_document")
        .header("X-API-KEY", key)
        .json(&document)
        .send()
        .await;

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
async fn test_index_document_group() {
    let key = "key";
    let req = reqwest::Client::new();

    for i in 0..10 {
        let content = format!("This is a test document {}", i);
        add_document(content, 10, i).await;
    }

    let make_group = GroupDocumentRequest { doc_group_size: 2 };

    // CREATE document
    let response = req
        .post("http://localhost:8090/api/document_group")
        .header("X-API-KEY", key)
        .json(&make_group)
        .send()
        .await;
    assert!(response.is_ok());
    assert!(response.unwrap().status() == 204);

    // First create document group
    let document_group = IndexDocumentGroupRequest::Story {
        story_id: 10,
        doc_group_size: 2,
    };

    // INDEX documents
    let response = req
        .put("http://localhost:8090/api/document_group")
        .header("X-API-KEY", key)
        .json(&document_group)
        .send()
        .await;
    assert!(response.is_ok());
    let res = response.unwrap();
    if res.status() != 204 {
        let error = res.json::<ErrorResponse>().await.unwrap();
        panic!("code {:?} {:}", error.error_code, error.message);
    }
}
