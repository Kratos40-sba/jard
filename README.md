# Jard (جرد) — The Zero-Hardware Barcode Bridge

Turn any Windows PC into a local Wi-Fi server and use your personal smartphones as high-speed, synchronized barcode scanners for inventory.

## 🚀 Vision
Jard eliminates the need for expensive dedicated barcode hardware. By bridging your phone's camera to your PC via local Wi-Fi, it creates a seamless, zero-install inventory experience.

## ✨ Features
- **Zero Installation**: Single `.exe` includes everything; the mobile app is served directly from the PC.
- **High Concurrency**: Multiple employees can scan simultaneously to the same PC.
- **Network Auto-Discovery**: Automatically finds your local IP and generates a pairing QR code.
- **Real-Time Synchronization**: Watch scans appear on your desktop as they happen.
- **Excel Export**: Clean, aggregated data ready for your POS software.

## 🛠 Tech Stack
- **Backend**: Rust, Axum, Tokio
- **Desktop UI**: egui / eframe
- **Mobile UI**: HTML5 + html5-qrcode
- **Exporting**: rust_xlsxwriter

## 📖 How it Works
1. **Launch**: Open `Jard.exe` on your PC.
2. **Connect**: Scan the displayed QR code with any smartphone.
3. **Scan**: Start scanning barcodes on the mobile web page.
4. **Export**: Once finished, click "Export to Excel" on your PC.

## 🚧 Development
```bash
# Clone the repository
git clone https://github.com/your-org/jard.git

# Run the project
cargo run
```

## ⚖️ License
MIT
