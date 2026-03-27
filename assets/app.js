// RAF (رف) — Standard Application Logic
const urlParams = new URLSearchParams(window.location.search);
const token = urlParams.get('token');

let selectedOrderId = null;
let ordersCache = [];

async function fetchOrders() {
    const resp = await fetch('/api/orders', { headers: { 'X-Jard-Token': token } });
    if (resp.status === 401) {
        document.body.innerHTML = "<h1>Accès Refusé</h1>";
        return;
    }
    const orders = await resp.json();
    ordersCache = orders;
    renderOrderList();
}

function renderOrderList() {
    const container = document.getElementById('active-orders');
    container.innerHTML = '';

    ordersCache.forEach(order => {
        const div = document.createElement('div');
        div.className = `glass-card ${selectedOrderId === order.id ? 'success-glow' : ''}`;
        div.style.cursor = 'pointer';
        div.style.marginBottom = '1rem';
        div.onclick = () => selectOrder(order.id);

        const totalItems = order.items.reduce((acc, i) => acc + i.target_qty, 0);
        const packedItems = order.items.reduce((acc, i) => acc + i.packed_qty, 0);
        const progress = Math.round((packedItems / totalItems) * 100);

        div.innerHTML = `
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.8rem;">
                <span style="font-weight: 800;">Ordre #${order.id}</span>
                <span class="pill ${progress === 100 ? 'pill-success' : 'pill-active'}">${progress}%</span>
            </div>
            <div style="background: rgba(255,255,255,0.05); height: 8px; border-radius: 4px; overflow: hidden;">
                <div style="background: var(--primary-gradient); width: ${progress}%; height: 100%; transition: width 0.5s;"></div>
            </div>
            <div style="margin-top: 0.5rem; font-size: 0.75rem; opacity: 0.6;">
                ${packedItems} / ${totalItems} articles scannés
            </div>
        `;
        container.appendChild(div);
    });
}

function selectOrder(id) {
    selectedOrderId = id;
    renderOrderList();
    renderQRCode(id);
}

async function renderQRCode(orderId) {
    const response = await fetch('/api/ip', { headers: { 'X-Jard-Token': token } });
    const data = await response.json();
    const ip = data.ip;
    
    const url = `https://${ip}:8080/scanner?token=${token}&order_id=${orderId}`;
    const container = document.getElementById('qrcode-container');
    container.innerHTML = '';
    
    // Using local qrcode-generator
    const qr = qrcode(0, 'M');
    qr.addData(url);
    qr.make();
    container.innerHTML = qr.createImgTag(4);
    
    const label = document.createElement('p');
    label.innerText = `Ordre: ${orderId}`;
    label.style.marginTop = '1rem';
    label.style.fontWeight = '700';
    container.appendChild(label);
}

// Order Import Logic
const dropzone = document.getElementById('dropzone');
dropzone.onclick = () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = (e) => handleFile(e.target.files[0]);
    input.click();
};

async function handleFile(file) {
    if (!file) return;
    const reader = new FileReader();
    reader.onload = async (e) => {
        try {
            const data = JSON.parse(e.target.result);
            // Expected format: { id: "123", items: [ [barcode, name, qty], ... ] }
            const resp = await fetch('/api/orders', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json', 'X-Jard-Token': token },
                body: JSON.stringify(data)
            });
            if (resp.ok) {
                fetchOrders();
            }
        } catch (err) {
            alert("Erreur de format JSON !");
        }
    };
    reader.readAsText(file);
}

// Table Sync
async function updateScanTable() {
    const resp = await fetch('/api/scans', { headers: { 'X-Jard-Token': token } });
    const data = await resp.json();
    const tbody = document.getElementById('scan-tbody');
    tbody.innerHTML = '';

    Object.entries(data).forEach(([barcode, info]) => {
        const tr = document.createElement('tr');
        tr.innerHTML = `
            <td><code>${barcode}</code></td>
            <td style="font-weight:700">${info.product_name}</td>
            <td><span class="pill pill-count">${info.count}</span></td>
            <td style="opacity:0.6">${info.last_worker}</td>
            <td>—</td>
        `;
        tbody.appendChild(tr);
    });
}

document.getElementById('export-btn').onclick = () => window.location.href = `/api/export?token=${token}`;

if (token) {
    fetchOrders();
    updateScanTable();
    setInterval(() => {
        fetchOrders();
        updateScanTable();
    }, 2000);
}
