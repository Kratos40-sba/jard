// Get token from URL
const urlParams = new URLSearchParams(window.location.search);
const token = urlParams.get('token');

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
    const tbody = document.getElementById('scan-tbody');
    tbody.innerHTML = '';

    Object.entries(data).forEach(([barcode, info]) => {
        const tr = document.createElement('tr');
        
        // Barcode
        const tdBarcode = document.createElement('td');
        tdBarcode.textContent = barcode;
        
        // Product Name (NEW)
        const tdProductName = document.createElement('td');
        tdProductName.textContent = info.product_name || "Inconnu";
        
        // Count
        const tdCount = document.createElement('td');
        const strongCount = document.createElement('strong');
        strongCount.textContent = info.count;
        tdCount.appendChild(strongCount);
        
        // Worker
        const tdWorker = document.createElement('td');
        tdWorker.textContent = info.last_worker;
        
        // Actions
        const tdActions = document.createElement('td');
        const btnDelete = document.createElement('button');
        btnDelete.textContent = "Supprimer";
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

// ... rest of functions ...
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

// CSV Import Logic
document.getElementById('import-csv-btn').onclick = () => {
    document.getElementById('csv-file-input').click();
};

document.getElementById('csv-file-input').onchange = (e) => {
    const file = e.target.files[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = async (event) => {
        const text = event.target.result;
        const lines = text.split('\n');
        const products = [];

        for (let i = 1; i < lines.length; i++) { // Skip header
            const [barcode, name] = lines[i].split(',').map(s => s.trim());
            if (barcode && name) {
                products.push({ barcode, name });
            }
        }

        if (products.length > 0) {
            const resp = await fetch('/api/products', {
                method: 'POST',
                headers: { 
                    'Content-Type': 'application/json',
                    'X-Jard-Token': token
                },
                body: JSON.stringify(products)
            });
            if (resp.ok) {
                alert(`${products.length} produits importés !`);
                updateList();
            } else {
                alert("Erreur lors de l'import : " + resp.status);
            }
        }
    };
    reader.readAsText(file);
};

document.getElementById('export-btn').onclick = async () => {
    window.location.href = `/api/export?token=${token}`;
};

// Initial calls
if (token) {
    getIP();
    renderQRCode();
    setInterval(updateList, 1000);
    updateList();
} else {
    document.body.innerHTML = "<h1>Erreur</h1><p>Le token de sécurité est manquant dans l'URL.</p>";
}
