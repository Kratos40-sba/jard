use axum::{
    extract::{Path, Request, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use dashmap::DashMap;
use crate::assets::Assets;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanRecord {
    pub count: u32,
    pub last_worker: String,
}

#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub barcode: String,
    pub worker: String,
}

pub struct AppState {
    pub scans: DashMap<String, ScanRecord>,
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
        .route("/scan/:barcode", delete(delete_scan))
        .route("/export", get(export_excel))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit_middleware));

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
    let addr = req.extensions().get::<ConnectInfo<SocketAddr>>().map(|ci| ci.0.ip());
    
    if let Some(ip) = addr {
        let now = chrono::Utc::now();
        let mut entry = state.rate_limiter.entry(ip).or_insert((now, 0));
        let (last_time, count) = entry.value_mut();

        if (now - *last_time).num_seconds() < 1 {
            if *count > 10 { // Limit to 10 req/sec
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
    let token = req.headers()
        .get("X-Jard-Token")
        .and_then(|h| h.to_str().ok());

    let token = match token {
        Some(t) => Some(t.to_string()),
        None => {
            #[derive(Deserialize)]
            struct AuthQuery { token: String }
            let query = req.uri().query().and_then(|q| serde_urlencoded::from_str::<AuthQuery>(q).ok());
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
    let my_local_ip = local_ip_address::local_ip().unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));
    Json(serde_json::json!({ "ip": my_local_ip.to_string() }))
}

async fn get_qrcode(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let my_local_ip = local_ip_address::local_ip().unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));
    let url = format!("http://{}:8080/scanner?token={}", my_local_ip, state.access_token);
    Json(serde_json::json!({ "url": url, "token": state.access_token }))
}

async fn receive_scan(
    State(state): State<SharedState>,
    Json(payload): Json<ScanRequest>,
) -> StatusCode {
    // 1. Input Validation
    if payload.barcode.len() > 64 || payload.worker.len() > 64 {
        return StatusCode::BAD_REQUEST;
    }
    if !payload.barcode.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return StatusCode::BAD_REQUEST;
    }

    // 2. Process Scan
    state.scans.entry(payload.barcode).and_modify(|r| {
        r.count += 1;
        r.last_worker = payload.worker.clone();
    }).or_insert(ScanRecord {
        count: 1,
        last_worker: payload.worker,
    });
    StatusCode::OK
}

async fn list_scans(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let map: std::collections::HashMap<String, ScanRecord> = state.scans.iter().map(|r| (r.key().clone(), r.value().clone())).collect();
    Json(serde_json::json!(map))
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
    worksheet.write_string(0, 1, "Count").unwrap();
    worksheet.write_string(0, 2, "Last Worker").unwrap();

    for (row, entry) in state.scans.iter().enumerate() {
        let (barcode, record) = (entry.key(), entry.value());
        worksheet.write_string((row + 1) as u32, 0, barcode).unwrap();
        worksheet.write_number((row + 1) as u32, 1, record.count as f64).unwrap();
        worksheet.write_string((row + 1) as u32, 2, &record.last_worker).unwrap();
    }

    let buffer = workbook.save_to_buffer().unwrap();
    
    Response::builder()
        .header(header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .header(header::CONTENT_DISPOSITION, "attachment; filename=\"jard_export.xlsx\"")
        .body(axum::body::Body::from(buffer))
        .unwrap()
}
