# Jard Architecture (Barcode Bridge)

## 1. Vision
Jard is a zero-hardware solution to turn any PC into a central inventory hub using smartphones as barcode scanners.

## 2. Component Architecture

### A. Core Backend (Rust)
- **Embedded Web Server**: `axum` + `tokio`. Serves the mobile web app and collects scan data.
- **Embedded Assets**: `rust-embed`. Packs HTML/JS directly into the binary for zero-install portability.
- **Network Discovery**: `local-ip-address`. Finds the PC's Wi-Fi IP to host the bridge.

### B. Desktop UI (egui)
- **Connection Hub**: Displays a large QR code generated via `qrcode` for mobile pairing.
- **Real-Time Feed**: A scrolling list of incoming scans from connected phones.
- **Data Aggregation**: In-memory `HashMap` to group duplicate scans and maintain counts.

### C. Mobile Frontend (HTML5)
- **Browser-Based Scanner**: Uses `html5-qrcode` to access the phone's camera.
- **Zero-Install**: Accessed via a QR code scan; no app store download required.

## 3. Data Flow
1. **Server Start**: Jard identifies the local IP and starts the web server.
2. **Pairing**: The user scans the desktop QR code with a phone.
3. **Scanning**: The phone scans a barcode; the JS sends an HTTP POST to the PC.
4. **Display**: The desktop UI updates immediately to show the new scan.
5. **Export**: The owner exports the aggregated data to Excel (`rust_xlsxwriter`).

## 4. Key Best Practices
- **Single-Binary Portability**: No external dependencies or folders needed by the user.
- **Concurrency**: Axum handles multiple simultaneous mobile connections.
- **Separation**: Clear boundary between GUI state, server state, and business logic.
