use crate::assets::Assets;
use axum::{
    extract::{Path, Request, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanRecord {
    pub count: u32,
    pub last_worker: String,
    pub is_anomaly: bool,
    pub anomaly_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderItem {
    pub barcode: String,
    pub name: String,
    pub target_qty: u32,
    pub packed_qty: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
    pub id: String,
    pub items: Vec<OrderItem>,
    pub status: String, // "Active", "Complete"
}

#[derive(Debug, Deserialize)]
pub struct OrderRequest {
    pub id: String,
    pub items: Vec<(String, String, u32)>, // (barcode, name, qty)
}

#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub barcode: String,
    pub worker: String,
    pub order_id: Option<String>,
}

pub struct AppState {
    pub scans: DashMap<String, ScanRecord>,
    pub orders: DashMap<String, Order>,
    pub product_lookup: DashMap<String, String>,
    pub access_token: String,
    pub rate_limiter: DashMap<std::net::IpAddr, (chrono::DateTime<chrono::Utc>, u32)>,
}

pub type SharedState = Arc<AppState>;

pub fn router(state: SharedState) -> Router {
    let api_routes = Router::new()
        .route("/ip", get(get_ip))
        .route("/qrcode", get(get_qrcode))
        .route("/scan", post(receive_scan))
        .route("/scans", get(list_scans))
        .route("/orders", get(list_orders))
        .route("/orders", post(create_order))
        .route("/scan/:barcode", delete(delete_scan))
        .route("/products", post(update_products))
        .route("/export", get(export_excel))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ));

    Router::new()
        .route("/", get(index))
        .route("/scanner", get(scanner))
        .route("/assets/:path", get(serve_asset))
        .nest("/api", api_routes)
        .with_state(state)
}

async fn rate_limit_middleware(
    State(state): State<SharedState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    use axum::extract::ConnectInfo;
    use std::net::SocketAddr;

    // In a real production app behind a proxy, we'd check X-Forwarded-For
    let addr = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip());

    if let Some(ip) = addr {
        let now = chrono::Utc::now();
        let mut entry = state.rate_limiter.entry(ip).or_insert((now, 0));
        let (last_time, count) = entry.value_mut();

        if (now - *last_time).num_seconds() < 1 {
            if *count > 10 {
                // Limit to 10 req/sec
                return Err(StatusCode::TOO_MANY_REQUESTS);
            }
            *count += 1;
        } else {
            *last_time = now;
            *count = 1;
        }
    }

    Ok(next.run(req).await)
}

async fn auth_middleware(
    State(state): State<SharedState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check Header first
    let token = req
        .headers()
        .get("X-Jard-Token")
        .and_then(|h| h.to_str().ok());

    let token = match token {
        Some(t) => Some(t.to_string()),
        None => {
            #[derive(Deserialize)]
            struct AuthQuery {
                token: String,
            }
            let query = req
                .uri()
                .query()
                .and_then(|q| serde_urlencoded::from_str::<AuthQuery>(q).ok());
            query.map(|q| q.token)
        }
    };

    match token {
        Some(t) if t == state.access_token => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

async fn index() -> impl IntoResponse {
    let asset = Assets::get("index.html").expect("index.html missing");
    Html(asset.data)
}

async fn scanner() -> impl IntoResponse {
    let asset = Assets::get("scanner.html").expect("scanner.html missing");
    Html(asset.data)
}

async fn serve_asset(Path(path): Path<String>) -> impl IntoResponse {
    match Assets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(axum::body::Body::from(content.data))
                .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Error").into_response())
        }
        None => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}

async fn get_ip() -> Json<serde_json::Value> {
    let my_local_ip = local_ip_address::local_ip()
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));
    Json(serde_json::json!({ "ip": my_local_ip.to_string() }))
}

async fn get_qrcode(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let my_local_ip = local_ip_address::local_ip()
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));
    let url = format!(
        "https://{}:8080/scanner?token={}",
        my_local_ip, state.access_token
    );
    Json(serde_json::json!({ "url": url, "token": state.access_token }))
}

async fn receive_scan(
    State(state): State<SharedState>,
    Json(payload): Json<ScanRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    // 1. Order Verification Mode (RAF Core)
    if let Some(order_id) = payload.order_id {
        if let Some(mut order) = state.orders.get_mut(&order_id) {
            let mut found = false;
            let mut over_packed = false;
            let mut item_name = "Inconnu".to_string();

            for item in order.items.iter_mut() {
                if item.barcode == payload.barcode {
                    item_name = item.name.clone();
                    if item.packed_qty < item.target_qty {
                        item.packed_qty += 1;
                        found = true;
                    } else {
                        over_packed = true;
                    }
                    break;
                }
            }

            if found {
                return (
                    StatusCode::OK,
                    Json(
                        serde_json::json!({ "status": "ok", "message": format!("Correct: {}", item_name) }),
                    ),
                );
            } else if over_packed {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "status": "error", "message": "Déjà complet!" })),
                );
            } else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "status": "error", "message": "ARTICLE INCORRECT" })),
                );
            }
        }
    }

    // 2. Simple Inventory Fallback (Legacy Jard)
    let is_unknown = !state.product_lookup.contains_key(&payload.barcode);
    state
        .scans
        .entry(payload.barcode.clone())
        .and_modify(|r| {
            r.count += 1;
            r.last_worker = payload.worker.clone();
        })
        .or_insert(ScanRecord {
            count: 1,
            last_worker: payload.worker,
            is_anomaly: is_unknown,
            anomaly_reason: if is_unknown {
                Some("Inconnu".to_string())
            } else {
                None
            },
        });

    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "ok", "message": "Scanné" })),
    )
}

async fn create_order(
    State(state): State<SharedState>,
    Json(payload): Json<OrderRequest>,
) -> StatusCode {
    let order = Order {
        id: payload.id.clone(),
        status: "Active".to_string(),
        items: payload
            .items
            .into_iter()
            .map(|(barcode, name, qty)| OrderItem {
                barcode,
                name,
                target_qty: qty,
                packed_qty: 0,
            })
            .collect(),
    };
    state.orders.insert(payload.id, order);
    StatusCode::CREATED
}

async fn list_orders(State(state): State<SharedState>) -> Json<Vec<Order>> {
    let orders: Vec<Order> = state.orders.iter().map(|e| e.value().clone()).collect();
    Json(orders)
}

#[derive(Debug, Deserialize)]
pub struct ProductMetadata {
    pub barcode: String,
    pub name: String,
}

async fn update_products(
    State(state): State<SharedState>,
    Json(payload): Json<Vec<ProductMetadata>>,
) -> StatusCode {
    for item in payload {
        state.product_lookup.insert(item.barcode, item.name);
    }
    StatusCode::OK
}

async fn list_scans(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let mut results = serde_json::Map::new();

    for entry in state.scans.iter() {
        let barcode = entry.key();
        let record = entry.value();
        let product_name = state
            .product_lookup
            .get(barcode)
            .map(|v| v.value().clone())
            .unwrap_or_else(|| "Produit inconnu".to_string());

        results.insert(
            barcode.clone(),
            serde_json::json!({
                "count": record.count,
                "last_worker": record.last_worker,
                "product_name": product_name,
                "is_anomaly": record.is_anomaly,
                "anomaly_reason": record.anomaly_reason
            }),
        );
    }

    Json(serde_json::Value::Object(results))
}

async fn delete_scan(State(state): State<SharedState>, Path(barcode): Path<String>) -> StatusCode {
    state.scans.remove(&barcode);
    StatusCode::NO_CONTENT
}

async fn export_excel(State(state): State<SharedState>) -> impl IntoResponse {
    use rust_xlsxwriter::*;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    worksheet.write_string(0, 0, "Barcode").unwrap();
    worksheet.write_string(0, 1, "Produit").unwrap();
    worksheet.write_string(0, 2, "Quantité").unwrap();
    worksheet.write_string(0, 3, "Dernier Opérateur").unwrap();

    for (row, entry) in state.scans.iter().enumerate() {
        let (barcode, record) = (entry.key(), entry.value());
        let product_name = state
            .product_lookup
            .get(barcode)
            .map(|v| v.value().clone())
            .unwrap_or_else(|| "Inconnu".to_string());

        worksheet
            .write_string((row + 1) as u32, 0, barcode)
            .unwrap();
        worksheet
            .write_string((row + 1) as u32, 1, &product_name)
            .unwrap();
        worksheet
            .write_number((row + 1) as u32, 2, record.count as f64)
            .unwrap();
        worksheet
            .write_string((row + 1) as u32, 2, &record.last_worker)
            .unwrap();
    }

    let buffer = workbook.save_to_buffer().unwrap();

    Response::builder()
        .header(
            header::CONTENT_TYPE,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"jard_export.xlsx\"",
        )
        .body(axum::body::Body::from(buffer))
        .unwrap()
}
