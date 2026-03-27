use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use dashmap::DashMap;
use raf::api::{router, AppState, Order, OrderItem, ScanRequest};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt; // for `oneshot`

#[tokio::test]
async fn test_receive_scan_logic() {
    // 1. Setup Mock State (using in-memory SQLite for testing if needed)
    // For now, let's test the response logic
    let token = "test-token".to_string();
    let db = Arc::new(raf::infra::db::Database::new(":memory:").unwrap());

    let state = Arc::new(AppState {
        scans: DashMap::new(),
        orders: DashMap::new(),
        product_lookup: DashMap::new(),
        db: db.clone(),
        access_token: token.clone(),
        rate_limiter: DashMap::new(),
    });

    // 2. Add a dummy order
    let order_id = "ORD-1".to_string();
    let order = Order {
        id: order_id.clone(),
        status: "Active".to_string(),
        items: vec![OrderItem {
            barcode: "123".to_string(),
            name: "Test Item".to_string(),
            target_qty: 2,
            packed_qty: 0,
        }],
    };
    db.save_order(&order).unwrap();
    state.orders.insert(order_id.clone(), order);

    let app = router(state);

    // 3. Test Correct Scan
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/scan")
                .header("X-Jard-Token", &token)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "barcode": "123",
                        "worker": "TestWorker",
                        "order_id": order_id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 4. Test Incorrect Scan (Article not in order)
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/scan")
                .header("X-Jard-Token", &token)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "barcode": "999",
                        "worker": "TestWorker",
                        "order_id": order_id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
