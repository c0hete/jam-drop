// jam-drop — UI MVP, vanilla JS.
// Por ahora: solo placeholder de "refrescar peers". El daemon llega en Fase 2.

const peersList = document.getElementById("peers-list");
const peersStatus = document.getElementById("peers-status");
const refreshBtn = document.getElementById("refresh-peers");

/**
 * MVP: lista de peers fija desde una config local (eventualmente desde el backend Tauri).
 * Cuando esté el daemon, esto se mueve a una llamada `invoke('list_peers')`.
 */
const peersStub = [
  { name: "pc",       ip: "10.10.0.2" },
  { name: "telefono", ip: "10.10.0.3" },
  { name: "laptop",   ip: "10.10.0.4" },
];

async function ping(_peer) {
  // Placeholder: cuando exista el daemon, esto hace fetch a http://<ip>:7777/ping con timeout corto.
  return false;
}

async function refresh() {
  peersStatus.textContent = "escaneando...";
  peersList.innerHTML = "";

  const results = await Promise.all(
    peersStub.map(async (p) => ({ ...p, alive: await ping(p) })),
  );

  peersStatus.textContent = `${results.filter((p) => p.alive).length} de ${results.length} vivos`;

  for (const p of results) {
    const li = document.createElement("li");
    li.innerHTML = `
      <span><strong>${p.name}</strong> <span class="muted">${p.ip}</span></span>
      <span class="${p.alive ? "ok" : "down"}">${p.alive ? "● vivo" : "○ sin respuesta"}</span>
    `;
    peersList.appendChild(li);
  }
}

refreshBtn.addEventListener("click", refresh);
refresh();
