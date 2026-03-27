// Get token from URL
const urlParams = new URLSearchParams(window.location.search);
const token = urlParams.get('token');

// Chart initialization
let velocityChart, workerChart;
const velocityData = [];
const velocityLabels = [];
let lastTotalScans = 0;

function initCharts() {
    const ctxVelocity = document.getElementById('velocity-chart').getContext('2d');
    velocityChart = new Chart(ctxVelocity, {
        type: 'line',
        data: {
            labels: velocityLabels,
            datasets: [{
                label: 'Scans/s',
                data: velocityData,
                borderColor: '#4f46e5',
                borderWidth: 2,
                pointRadius: 0,
                tension: 0.4,
                fill: true,
                backgroundColor: 'rgba(79, 70, 229, 0.05)'
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: { 
                y: { beginAtZero: true, grid: { color: '#f3f4f6' }, ticks: { font: { size: 10 } } },
                x: { grid: { display: false }, ticks: { display: false } }
            },
            plugins: { title: { display: true, text: 'VITESSE DE SCAN', align: 'start', color: '#6b7280', font: { weight: '800', size: 11 } }, legend: { display: false } }
        }
    });

    const ctxWorker = document.getElementById('worker-chart').getContext('2d');
    workerChart = new Chart(ctxWorker, {
        type: 'bar',
        data: {
            labels: [],
            datasets: [{
                label: 'Scans',
                data: [],
                backgroundColor: '#111827',
                borderRadius: 4,
                barThickness: 20
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: { 
                y: { beginAtZero: true, grid: { color: '#f3f4f6' }, ticks: { font: { size: 10 } } },
                x: { grid: { display: false } }
            },
            plugins: { title: { display: true, text: 'PERFORMANCE', align: 'start', color: '#6b7280', font: { weight: '800', size: 11 } }, legend: { display: false } }
        }
    });
}

function updateAnalytics(data) {
    // 1. Update Worker Chart
    const workerCounts = {};
    let totalCurrentScans = 0;

    Object.values(data).forEach(info => {
        const worker = info.last_worker || "Inconnu";
        workerCounts[worker] = (workerCounts[worker] || 0) + info.count;
        totalCurrentScans += info.count;
    });

    workerChart.data.labels = Object.keys(workerCounts);
    workerChart.data.datasets[0].data = Object.values(workerCounts);
    workerChart.update();

    // 2. Update Velocity Chart
    if (lastTotalScans === 0 && totalCurrentScans > 0) {
        lastTotalScans = totalCurrentScans;
        return; // Skip the first data point to avoid a massive spike
    }
    const delta = totalCurrentScans - lastTotalScans;
    lastTotalScans = totalCurrentScans;

    const now = new Date().toLocaleTimeString();
    velocityData.push(delta);
    velocityLabels.push(now);

    if (velocityData.length > 20) {
        velocityData.shift();
        velocityLabels.shift();
    }
    velocityChart.update();
}

async function updateList() {
    if (!token) return;
    const response = await fetch('/api/scans', {
        headers: { 'X-Jard-Token': token }
    });
    if (response.status === 401) {
        document.body.innerHTML = "<h1>401 Accès Refusé</h1><p>Token invalide ou manquant.</p>";
        return;
    }
    const data = await response.json();
    
    // Update Analytics
    if (!velocityChart) initCharts();
    updateAnalytics(data);

    const tbody = document.getElementById('scan-tbody');
    tbody.innerHTML = '';

    Object.entries(data).forEach(([barcode, info]) => {
        const tr = document.createElement('tr');
        const tdBarcode = document.createElement('td');
        tdBarcode.innerHTML = `<code style="background:#f3f4f6;padding:2px 4px;border-radius:4px">${barcode}</code>`;
        const tdProductName = document.createElement('td');
        tdProductName.textContent = info.product_name || "Inconnu";
        tdProductName.style.fontWeight = "600";
        
        const tdCount = document.createElement('td');
        const pillCount = document.createElement('span');
        pillCount.className = "pill pill-count";
        pillCount.textContent = info.count;
        tdCount.appendChild(pillCount);
        
        const tdWorker = document.createElement('td');
        tdWorker.textContent = info.last_worker;
        tdWorker.style.color = "#6b7280";
        
        const tdActions = document.createElement('td');
        const btnDelete = document.createElement('button');
        btnDelete.textContent = "Supprimer";
        btnDelete.className = "btn btn-danger btn-sm"; // btn-sm is a hint for me to style it small
        btnDelete.style.fontSize = "0.75rem";
        btnDelete.onclick = () => deleteScan(barcode);
        tdActions.appendChild(btnDelete);

        tr.appendChild(tdBarcode);
        tr.appendChild(tdProductName);
        tr.appendChild(tdCount);
        tr.appendChild(tdWorker);
        tr.appendChild(tdActions);
        tbody.appendChild(tr);
    });
}

// ... rest of same functions (getIP, renderQRCode, deleteScan, CSV Import) ...
async function getIP() {
    if (!token) return;
    const response = await fetch('/api/ip', {
        headers: { 'X-Jard-Token': token }
    });
    const data = await response.json();
    document.getElementById('local-ip').innerText = `Serveur local à l'adresse http://${data.ip}:8080`;
}

async function renderQRCode() {
    if (!token) return;
    const response = await fetch('/api/qrcode', {
        headers: { 'X-Jard-Token': token }
    });
    const data = await response.json();
    new QRCode(document.getElementById("qrcode-container"), {
        text: data.url,
        width: 256,
        height: 256
    });
}

async function deleteScan(barcode) {
    await fetch(`/api/scan/${barcode}`, { 
        method: 'DELETE',
        headers: { 'X-Jard-Token': token }
    });
    updateList();
}

document.getElementById('import-csv-btn').onclick = () => {
    document.getElementById('csv-file-input').click();
};
document.getElementById('import-csv-btn').className = "btn btn-secondary";

document.getElementById('csv-file-input').onchange = (e) => {
    const file = e.target.files[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = async (event) => {
        const text = event.target.result;
        const lines = text.split('\n');
        const products = [];
        for (let i = 1; i < lines.length; i++) {
            const [barcode, name] = lines[i].split(',').map(s => s.trim());
            if (barcode && name) products.push({ barcode, name });
        }
        if (products.length > 0) {
            const resp = await fetch('/api/products', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json', 'X-Jard-Token': token },
                body: JSON.stringify(products)
            });
            if (resp.ok) { alert(`${products.length} produits importés !`); updateList(); }
        }
    };
    reader.readAsText(file);
};

document.getElementById('export-btn').onclick = async () => {
    window.location.href = `/api/export?token=${token}`;
};
document.getElementById('export-btn').className = "btn btn-primary";

if (token) {
    getIP();
    renderQRCode();
    // Initialize charts immediately with empty data to avoid "weird" loading later
    if (!velocityChart) initCharts();
    updateList(); 
    setInterval(updateList, 1000);
} else {
    document.body.innerHTML = "<h1>Erreur</h1><p>Le token de sécurité est manquant dans l'URL.</p>";
}
