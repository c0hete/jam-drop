# PLAN — jam-drop MVP

> Vivo. Se va marcando a medida que avanzamos. Cuando una fase cambie de scope,
> se actualiza acá antes de tocar código.

## MVP — qué debe hacer (mínimo viable)

1. Cada equipo corre un **daemon** que escucha HTTP en `JAMDROP_BIND_IP:JAMDROP_PORT`
   (default `10.10.0.x:7777`).
2. Endpoints mínimos:
   - `GET /ping` → responde `pong` + nombre del equipo + versión (heartbeat).
   - `POST /upload` → recibe un archivo y lo guarda en el `inbox` local.
3. Una **UI** local (webview de Tauri) que:
   - Lee `peers.toml` (lista de equipos conocidos).
   - Hace `GET /ping` a cada uno → muestra los **vivos** en un dropdown.
   - Permite seleccionar archivo + destino + enviar (`POST /upload` al elegido).
4. **Auth simple**: header `X-Token: $JAMDROP_SHARED_TOKEN` (mismo token en todos los
   equipos, en `.env`). Suficiente para MVP. v2: mTLS o llaves por peer.
5. Inbox: en data dir del usuario (`%APPDATA%\jam-drop\inbox\` en Win,
   `~/.local/share/jam-drop/inbox/` en Linux, equivalente en Mac).

## Fuera del MVP (v2+)

- Icono en system tray con drag-and-drop nativo (overlay).
- Autodescubrimiento por broadcast UDP.
- mTLS o pares de llaves por peer.
- Cliente móvil (Android via Tauri Mobile).
- Selector de carpeta destino al recibir.
- Historial de envíos/recibidos.
- Notificación al recibir.

## Fases de ejecución

### Fase 0 — Andamiaje (esta fase)
- [x] Carpeta + estructura
- [x] `.mc`, `.gitignore`, `.env.example`, `CLAUDE.md`, `README.md`, `PLAN.md`
- [ ] Pre-commit hook con gitleaks
- [ ] `git init` + primer commit local
- [ ] Workflows `ci.yml` + `release.yml` esqueleto
- [ ] Verificación gitleaks
- [ ] Repo en GitHub (privado primero)

### Fase 1 — Toolchain
- [ ] Instalar Rust (`rustup`) si no está
- [ ] Instalar Tauri CLI (`cargo install tauri-cli --version "^2.0"`)
- [ ] Scaffold inicial con `cargo create-tauri-app` template vanilla
- [ ] Confirmar que `cargo tauri dev` arranca

### Fase 2 — Daemon mínimo
- [ ] `daemon.rs` con `axum` corriendo en `JAMDROP_BIND_IP:JAMDROP_PORT`
- [ ] `GET /ping` con auth
- [ ] `POST /upload` que guarda en `inbox/`
- [ ] Test `cargo test` del upload
- [ ] CI verde con tests

### Fase 3 — UI mínima
- [ ] HTML con `<input type="file">` + `<select>` de peers + botón
- [ ] JS que llama a `/ping` de cada peer y filtra vivos
- [ ] Envío del archivo con `fetch`
- [ ] Indicador visual: enviando / OK / error

### Fase 4 — Prueba real PC ↔ laptop
- [ ] `peers.toml` con PC y laptop
- [ ] Token compartido generado
- [ ] Pasar un archivo PC → laptop por la malla
- [ ] Pasar un archivo laptop → PC

### Fase 5 — Release v0.1.0
- [ ] Tag `v0.1.0` en git
- [ ] `release.yml` compila para Win/Linux/Mac
- [ ] Binarios suben al GitHub Release
- [ ] Probar el binario release en laptop

### Fase 6 — Hacer público
- [ ] Revisar manualmente que no hay nada sensible en el repo
- [ ] Cambiar el repo a `public` en GitHub
- [ ] Linkearlo desde el portafolio (alvaradomazzei.cl)

## Decisiones registradas

| Cuándo | Decisión | Razón |
|---|---|---|
| 2026-06-11 | Tauri (vs Python/Go) | Aprender Rust, binario chico, futuro móvil |
| 2026-06-11 | Auth por token compartido en MVP | Suficiente para 3-4 equipos propios; mTLS es v2 |
| 2026-06-11 | Descubrimiento por `peers.toml` + ping | Más simple que broadcast UDP; suficiente para n=4 |
| 2026-06-11 | Inbox en data dir (no Desktop) | Prolijo; el usuario lo abre desde la UI |
| 2026-06-11 | Repo público al final, no al inicio | Rieles primero, exposición después |
