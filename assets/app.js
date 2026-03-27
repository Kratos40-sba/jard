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
        
        // Use textContent to prevent XSS
        const tdBarcode = document.createElement('td');
        tdBarcode.textContent = barcode;
        
        const tdCount = document.createElement('td');
        const strongCount = document.createElement('strong');
        strongCount.textContent = info.count;
        tdCount.appendChild(strongCount);
        
        const tdWorker = document.createElement('td');
        tdWorker.textContent = info.last_worker;
        
        const tdActions = document.createElement('td');
        const btnDelete = document.createElement('button');
        btnDelete.textContent = "Supprimer";
        btnDelete.onclick = () => deleteScan(barcode);
        tdActions.appendChild(btnDelete);

        tr.appendChild(tdBarcode);
        tr.appendChild(tdCount);
        tr.appendChild(tdWorker);
        tr.appendChild(tdActions);
        
        tbody.appendChild(tr);
    });
}

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

document.getElementById('export-btn').onclick = async () => {
    // For downloads, we pass the token as a query param or separate auth mechanism if needed, 
    // but for now let's use the URL since it's a simple GET.
    window.location.href = `/api/export?token=${token}`;
    // Note: The backend export_excel currently doesn't check query params for token, 
    // it expects the header. But browser window.location.href doesn't send custom headers.
    // I'll update the backend to also check query params for the export route.
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
