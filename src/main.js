// jam-drop — UI MVP, vanilla JS.
// Llama a comandos Tauri del backend Rust (no fetch directo al daemon).
// Esa es la frontera: el JS solo orquesta UI; toda la red la hace Rust.

const { invoke } = window.__TAURI__.core;

// ----- referencias del DOM -----
const meLine = document.getElementById("me-line");
const peersList = document.getElementById("peers-list");
const peersStatus = document.getElementById("peers-status");
const peersEmptyHint = document.getElementById("peers-empty-hint");
const peersFileLink = document.getElementById("peers-file-link");
const refreshBtn = document.getElementById("refresh-peers");

const sendForm = document.getElementById("send-form");
const fileInput = document.getElementById("file-input");
const destSelect = document.getElementById("dest-select");
const sendBtn = document.getElementById("send-btn");
const sendStatus = document.getElementById("send-status");

// estado local en memoria
let lastAlivePeers = [];

// ----- identidad: quien soy yo -----
async function loadMe() {
  try {
    const me = await invoke("me");
    const tokenWarn = me.has_token ? "" : " · ⚠ sin token";
    meLine.textContent = `soy "${me.device_name}" · ${me.bind_ip}:${me.port}${tokenWarn}`;
    // mostrar ruta del peers.toml para que el usuario sepa dónde editarlo
    try {
      const path = await invoke("peers_file_path");
      peersFileLink.textContent = path;
    } catch (_) {
      /* ignorar */
    }
  } catch (e) {
    meLine.textContent = `error cargando identidad: ${e}`;
  }
}

// ----- refrescar lista de peers + estado vivo/muerto -----
async function refresh() {
  peersStatus.textContent = "escaneando...";
  peersList.innerHTML = "";
  peersEmptyHint.classList.add("hidden");

  let statuses;
  try {
    statuses = await invoke("check_peers");
  } catch (e) {
    peersStatus.textContent = `error: ${e}`;
    return;
  }

  if (!statuses || statuses.length === 0) {
    peersStatus.textContent = "no hay peers configurados";
    peersEmptyHint.classList.remove("hidden");
    rebuildDestOptions([]);
    return;
  }

  const alive = statuses.filter((p) => p.alive);
  peersStatus.textContent = `${alive.length} de ${statuses.length} vivos`;

  for (const p of statuses) {
    const li = document.createElement("li");
    const latency = p.alive && p.latency_ms != null ? ` · ${p.latency_ms} ms` : "";
    li.innerHTML = `
      <span><strong>${escapeHtml(p.name)}</strong>
        <span class="muted">${escapeHtml(p.ip)}:${p.port}${latency}</span>
      </span>
      <span class="${p.alive ? "ok" : "down"}">${p.alive ? "● vivo" : "○ sin respuesta"}</span>
    `;
    peersList.appendChild(li);
  }

  lastAlivePeers = alive;
  rebuildDestOptions(alive);
}

function rebuildDestOptions(alive) {
  // preservar la selección previa si el peer sigue vivo
  const prev = destSelect.value;
  destSelect.innerHTML = "";
  const placeholder = document.createElement("option");
  placeholder.value = "";
  placeholder.textContent = alive.length
    ? "— elegí un equipo vivo —"
    : "— sin equipos vivos —";
  destSelect.appendChild(placeholder);
  for (const p of alive) {
    const opt = document.createElement("option");
    opt.value = p.name;
    opt.textContent = `${p.name} (${p.ip})`;
    destSelect.appendChild(opt);
  }
  if (prev && alive.some((p) => p.name === prev)) {
    destSelect.value = prev;
  }
}

// ----- enviar archivo -----
sendForm.addEventListener("submit", async (e) => {
  e.preventDefault();
  const file = fileInput.files[0];
  const dest = destSelect.value;
  if (!file || !dest) return;

  sendBtn.disabled = true;
  sendStatus.textContent = `enviando ${file.name} a ${dest}...`;
  try {
    const data = new Uint8Array(await file.arrayBuffer());
    // Tauri serializa Uint8Array como Vec<u8>
    const savedAs = await invoke("send_file", {
      peerName: dest,
      filename: file.name,
      data: Array.from(data),
    });
    sendStatus.innerHTML = `<span class="ok">✓ enviado como <code>${escapeHtml(savedAs)}</code></span>`;
    fileInput.value = "";
  } catch (err) {
    sendStatus.innerHTML = `<span class="down">✗ falló: ${escapeHtml(String(err))}</span>`;
  } finally {
    sendBtn.disabled = false;
  }
});

// ----- helpers -----
function escapeHtml(s) {
  return String(s)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

// ----- arranque -----
refreshBtn.addEventListener("click", refresh);
loadMe();
refresh();
// refresco automático cada 10s para reflejar peers que se conectan/desconectan
setInterval(refresh, 10000);
