# Avance y pendientes — jam-drop

> Snapshot del estado del proyecto al **cierre de la sesión 2026-06-11**.
> Este doc es lo primero que se lee al retomar. Estado granular, no narrativo.

## Estado actual: ✅ v0.1.0 funcional

La app **mueve archivos de verdad** entre los equipos de la malla WireGuard.
Probado en vivo PC ↔ laptop con la red real (mailcow NY como hub) y archivo
recibido confirmado en `inbox/` de la laptop.

## ✅ Hecho

### Fase 0 — Andamiaje y rieles
- [x] Estructura del proyecto en `JRAM/apps/jam-drop/`.
- [x] Andamiaje del paraguas: `.mc`, `.gitignore`, `.env.example`, `CLAUDE.md`, `README.md`.
- [x] `.gitleaks.toml` con allowlist justificada (`.env.example` y `peers.example.toml`).
- [x] Pre-commit hook (`.githooks/pre-commit`) corre **gitleaks + cargo fmt + cargo clippy**
  con fallback a `~/.cargo/bin` si `cargo` no está en `PATH` del shell del hook.
- [x] `.github/workflows/ci.yml` — gitleaks + Rust fmt/clippy/test, condicional a que
  exista `src-tauri/`.
- [x] `.github/workflows/release.yml` — Tauri action multi-OS (Win + macOS + Linux),
  dispara con tag `v*`, publica directo (no draft) con body informativo.
- [x] Repo público en GitHub: **https://github.com/c0hete/jam-drop**.
- [x] CI verde en cada commit a `main`.

### Fase 1 — Toolchain y scaffold
- [x] Rust toolchain instalado: `rustc 1.96.0`, `cargo 1.96.0` (via winget Rustup).
- [x] MSVC Build Tools 2022 con workload C++ + Windows 11 SDK (~3 GB en disco).
- [x] Tauri CLI 2.11.2 instalado (`cargo install tauri-cli`).
- [x] Scaffold con `cargo tauri init` integrado al proyecto existente.
- [x] Ajustes del scaffold: `Cargo.toml` con nombre `jam-drop`, `lib = "jam_drop_lib"`,
  perfil `release` con LTO + opt-size + strip → binario de ~4.7 MB.
- [x] `tauri.conf.json` con `identifier = "cl.alvaradomazzei.jamdrop"` y
  `withGlobalTauri: true` (necesario para JS vanilla).

### Fase 2 — Daemon HTTP
- [x] `src/config.rs`: lee `.env` con `dotenvy`, defaults seguros (loopback,
  sin token = no acepta uploads). Inbox por default en `dirs::data_dir()/jam-drop/inbox`.
- [x] `src/daemon.rs`: router axum con:
  - `GET /ping` → JSON `{name, version}`, **sin auth** (para descubrimiento).
  - `POST /upload` → multipart con campo `file`, requiere `X-Token` igual al
    `JAMDROP_SHARED_TOKEN`. Sanitiza nombres (anti path-traversal), auto-rename
    si colisiona (`doc (1).txt`, etc.).
- [x] `src/lib.rs`: spawnea el daemon en runtime tokio antes de abrir Tauri.
  Si el daemon falla al bindear, loguea pero igual abre la UI.
- [x] **13 tests passing** (10 daemon + 2 peers + 1 config). En CI Linux también.
  Test `safe_filename_descarta_rutas_windows` con `#[cfg(windows)]` (los `\` solo
  son separadores en Windows).
- [x] Modo seguro: si `JAMDROP_SHARED_TOKEN` está vacío → todos los uploads devuelven
  401, incluso con cualquier token. Evita un servidor abierto por accidente.

### Fase 3 — UI conectada
- [x] `src/peers.rs`: lee `peers.toml` del data dir, pingeo concurrente con
  timeout de 1.5s, envío multipart con auth. Si el archivo no existe lo crea
  con placeholder comentado.
- [x] `src/lib.rs`: 5 comandos Tauri expuestos al frontend con `invoke()`:
  - `me()` → identidad propia (device_name, bind_ip, port, inbox, has_token).
  - `list_peers()` → lectura cruda del TOML.
  - `check_peers()` → ping concurrente, devuelve `PeerStatus[]`.
  - `send_file(peer_name, filename, data)` → encapsula el envío.
  - `peers_file_path()` → ruta del peers.toml para mostrar en UI.
- [x] `src/index.html` + `src/main.js` + `src/style.css`:
  - Header con identidad (`soy "<nombre>" · <ip>:<puerto>`).
  - Card de peers con auto-refresh cada 10s (sin parpadeo — replace atómico).
  - Formulario de envío: file picker + dropdown destino + botón.
  - Botón "Refrescar" con estilo `.ghost` para no competir con el primario.
  - Indicador discreto de "escaneando" con `::after` animado.
  - Escape HTML para todo lo que viene del backend.

### Fase 4 — Configuración y prueba real
- [x] Token compartido generado (64 chars hex, vive solo en cada `.env`).
- [x] `.env` del PC (`JAMDROP_DEVICE_NAME=pc`, `BIND_IP=10.10.0.2`, token).
- [x] `peers.toml` del PC con los 3 equipos (`pc`, `laptop`, `telefono`).
- [x] **Test de humo**: PC se envió un archivo a sí mismo (loopback por wg0) — funcionó.
- [x] `cargo tauri build` → instaladores `.msi` (2.5 MB) y `.exe` setup NSIS (1.9 MB)
  + binario standalone (4.7 MB).
- [x] **Instalación local en el PC**: binario + `.env` + ícono copiados a
  `%LOCALAPPDATA%\Programs\jam-drop\`. Shortcut en el escritorio.
- [x] **Kit de transporte para laptop** en `D:\jam-drop-laptop\` (con `.env`,
  `peers.toml`, `.msi`, `LEEME.md` con pasos + diagnóstico).
- [x] **Instalación en laptop** + creación de `.env` y `peers.toml`.
- [x] **PRUEBA FINAL**: archivo enviado desde PC → llegó al `inbox` de la laptop.

### Fase 5 — Release oficial v0.1.0
- [x] Tag `v0.1.0` empujado a GitHub → `release.yml` disparado.
- [x] Workflow compilando en Windows + macOS + Linux (corriendo al cierre de sesión).
- [x] Release URL prevista: https://github.com/c0hete/jam-drop/releases/tag/v0.1.0

## 🔜 Pendiente para próximas sesiones

### Próxima sesión recomendada: **auto-updater integrado**

El usuario preguntó "¿cómo actualizo la app?" al final de la sesión. La respuesta para
escalar es agregar el plugin oficial de Tauri. Plan:

1. **Generar par de llaves de firma** con `cargo tauri signer generate`. La privada va
   a GitHub Secrets (`TAURI_SIGNING_PRIVATE_KEY`). La pública se embede en el binario.
2. **Agregar `tauri-plugin-updater`** al `Cargo.toml` + plugin en `lib.rs`.
3. **Configurar feed** en `tauri.conf.json`: `plugins.updater.endpoints` apuntando a
   `https://github.com/c0hete/jam-drop/releases/latest/download/latest.json` (Tauri
   action genera ese JSON automáticamente cuando firmás).
4. **Actualizar `release.yml`**: usar las env vars `TAURI_SIGNING_PRIVATE_KEY` y
   `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` para firmar.
5. **Handler en `lib.rs`**: en `setup()`, chequear update, si hay nuevo mostrar dialog
   "actualizar?" → descarga + reinicio.
6. **Probar el flujo**: tagear `v0.1.1` (un cambio cosmético) → en PC/laptop debería
   aparecer aviso de update.

Costo estimado: 1 sesión dedicada (~1-2 hs).

### v0.2 — mejoras de UX

- [ ] **System tray**: ícono al lado del reloj, app sigue corriendo aunque cierres
  la ventana. Click derecho → abrir / cerrar / inbox.
- [ ] **Drag-and-drop nativo**: arrastrar un archivo desde el Explorer a la ventana
  y que aparezca preseleccionado en el form. Usar `tauri::event::on_window_event`.
- [ ] **Notificación al recibir**: toast del sistema "te llegó X de Y".
- [ ] **Ver inbox desde la UI**: tabla con lo recibido + botón "abrir carpeta".

### v0.3 — robustez de red

- [ ] **Descubrimiento por broadcast UDP**: que no haga falta editar `peers.toml`.
  Cada equipo manda anuncios en `<malla>/24` y se descubren solos.
- [ ] **Reintentos con backoff** en envío.
- [ ] **Soporte para múltiples archivos** en un solo upload (zip al vuelo).
- [ ] **mTLS o llaves por peer** en vez de token compartido (cada peer firma).

### v0.4 — móvil

- [ ] **Tauri Mobile** (Android primero). Mismo código, target distinto.
- [ ] Adaptar UI para pantalla chica.

### Backlog de calidad técnica

- [ ] **Ícono propio** (hoy son los placeholder de Tauri — "dos círculos").
- [ ] **Tests E2E** con WebDriver: arrancar app, click en form, verificar inbox.
- [ ] **Métricas básicas**: contador de archivos enviados/recibidos, último error.
- [ ] **Logs persistidos** en archivo además de stdout, con rotación.
- [ ] **Hot-reload del `.env`**: cambiar sin reiniciar la app.
- [ ] **CHANGELOG.md** mantenido con cada tag.

### Mejoras a CI/CD

- [ ] **Actualizar `actions/checkout`** a v5 (GitHub avisa que v4 usa Node 20
  deprecado a partir de 2026-06-16).
- [ ] **Tests en CI también para Windows + macOS**, no solo Linux (hoy solo Linux
  corre fmt+clippy+test; los OS sólo se ejercitan en `release.yml`).
- [ ] **Cobertura de tests** con `cargo-llvm-cov` reportada en PR.

## 📌 Decisiones tomadas (resumen)

| Cuándo | Decisión | Razón corta |
|---|---|---|
| 2026-06-11 | Tauri + Rust + UI vanilla | Aprender Rust, binario chico, futuro móvil |
| 2026-06-11 | axum + tokio para el daemon | Estándar Rust, async nativo |
| 2026-06-11 | Frontend llama a `invoke()`, no `fetch` directo | Más seguro, mismo path en móvil |
| 2026-06-11 | Token compartido en `.env` (no mTLS) | MVP simple; mTLS a v0.3 |
| 2026-06-11 | `peers.toml` manual (no broadcast) | MVP simple; broadcast a v0.3 |
| 2026-06-11 | Inbox en `data_dir` (no Desktop) | Prolijo; xplatform; el usuario lo abre desde UI |
| 2026-06-11 | Repo público desde el inicio | Portafolio; rieles anti-leak fuertes mitigan riesgo |
| 2026-06-11 | `withGlobalTauri: true` | UI vanilla sin bundler — `window.__TAURI__` global |
| 2026-06-11 | Release directo (no draft) | Sesión de "publicar" innecesaria para uso personal |

## 🧠 Gotchas conocidos (ahorrar tiempo al retomar)

1. **`cargo fmt --check` falla con código generado**. El scaffold de Tauri usa
   indent de 2 espacios; rustfmt default espera 4. Aplicar `cargo fmt --all`
   cualquier vez que `cargo tauri init` agregue algo. El pre-commit hook ya lo
   cubre, pero CI también lo chequea.

2. **El `.env` se lee desde el CWD al momento de arrancar**. Por eso el shortcut
   del escritorio apunta a `%LOCALAPPDATA%\Programs\jam-drop\` con `WorkingDirectory`
   ahí (donde está el `.env`). Si lanzás `jam-drop.exe` desde otro lado, no carga.

3. **`reqwest` sin `default-features`**: usar `rustls-tls` y NO `native-tls`, para
   no depender de OpenSSL del sistema en CI Linux. Si agregás `reqwest` a otro
   crate del workspace, mismo cuidado.

4. **`Path::new(...).file_name()` se comporta distinto Win/Linux**: en Linux `\`
   no es separador. Test de path-traversal con `\` va `#[cfg(windows)]`.

5. **`safe_filename` descarta paths absolutos**: si el frontend manda `C:\foo\bar.txt`
   solo se guarda `bar.txt`. El nombre original NO se preserva con su path.

6. **`withGlobalTauri: true` es obligatorio para JS vanilla**. Sin esto,
   `window.__TAURI__` no existe y `invoke()` falla con "undefined".

7. **El daemon escucha en `JAMDROP_BIND_IP`, no en `0.0.0.0`**. Si en otro PC con
   `.env` que diga `127.0.0.1` querés alcanzar al daemon desde la malla, no anda.
   La IP debe ser la de la interfaz de WireGuard local.

8. **El servicio `WireGuardTunnel$<nombre>` se hace "huérfano" si borrás el
   archivo `.conf.dpapi`** (Program Files / WireGuard / Data / Configurations).
   El servicio queda registrado pero la GUI no lo muestra. Si pasa: desinstalar
   con `wireguard.exe /uninstalltunnelservice <nombre>` y reinstalar desde la GUI.

9. **`cargo test` puede ser flaky en paralelo con env vars compartidas**. Los tests
   que mutan `std::env::remove_var` requieren `unsafe` en Rust 2024 y son
   inherentemente racy. Mantener uno solo o usar `serial_test`.

10. **El identifier de Tauri (`cl.alvaradomazzei.jamdrop`) no debe cambiar tras la
    primera instalación de un equipo**. Si cambia, Windows trata la app como
    distinta y queda duplicada en "Apps instaladas".
