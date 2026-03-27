use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use jard::api;
use tower::ServiceExt; // for `oneshot`

#[tokio::test]
async fn test_health_check() {
    let app = api::router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_secret_success() {
    let app = api::router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/secrets/db_secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_secret_not_found() {
    let app = api::router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/secrets/unknown_id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
