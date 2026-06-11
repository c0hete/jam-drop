# jam-drop — instrucciones del proyecto

App propia para mover archivos entre los equipos de la malla WireGuard sin nube ajena.
Filosofía: pequeño, hackeable, sobre tu propia red privada.

## Para qué

Drag-and-drop (en v2; MVP usa formulario) de archivos entre PC ↔ laptop ↔ teléfono
sobre la malla `10.10.0.0/24`. Cada equipo corre un daemon que recibe en una carpeta
inbox propia. Una UI permite elegir archivo y destino.

## Stack

- **Backend (daemon)**: Rust + [axum](https://github.com/tokio-rs/axum) + [tokio](https://tokio.rs/) + [serde](https://serde.rs/)
- **Frontend (UI)**: HTML + CSS + JS vanilla (sin frameworks en MVP)
- **Empaquetado**: [Tauri 2.x](https://tauri.app/)
- **Config**: `.env` (gitignored) + `peers.toml` (lista de equipos, en data dir)
- **Tests**: `cargo test` desde el inicio
- **CI/CD**: GitHub Actions — `ci.yml` (lint+test+gitleaks) y `release.yml` (binarios multi-OS)

## Estructura (planeada)

```
jam-drop/
├── .mc, .gitignore, .env.example, README, CLAUDE.md, PLAN.md
├── .github/workflows/  ← ci.yml + release.yml
├── docs/               ← decisiones arquitectónicas
├── src-tauri/          ← código Rust + config Tauri
│   ├── src/
│   │   ├── main.rs
│   │   ├── daemon.rs   ← /ping, /upload
│   │   ├── peers.rs    ← lista y heartbeat
│   │   └── config.rs   ← lee .env
│   └── Cargo.toml
└── src/                ← UI (HTML/CSS/JS)
    ├── index.html
    ├── style.css
    └── main.js
```

## Rieles anti-leak (NO mover)

- **`.env` JAMÁS se commitea.** Solo el `.env.example` (sin secretos reales).
- **Pre-commit hook**: `gitleaks` bloquea cualquier commit con secretos.
- **CI**: gitleaks corre en cada push/PR.
- **Pasar siempre por variables de entorno** lo que sea: tokens, llaves, IPs propias.

## Convenciones

- Commits sin atribución a IA (hay hook precommit que lo bloquea).
- Cambios visuales: preview antes de pushear.
- Antes de pushear cualquier cosa: `cargo fmt` + `cargo clippy` + `cargo test` + `gitleaks detect`.
- Repo en `c0hete/jam-drop` (privado al inicio, público tras revisión).

## Gotchas conocidos (a medida que aparezcan)

(Vacío por ahora. Si algo sorprende, anotarlo acá con su workaround.)

## Para profundizar

- Plan paso a paso: `PLAN.md`
- Decisiones: `docs/`
- Malla WireGuard: `JRAM/infraestructura/mail-alvaradomazzei/documentacion/WIREGUARD_MALLA.md`
