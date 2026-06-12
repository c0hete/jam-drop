# Tecnologías del stack — jam-drop

> Qué usamos y por qué. Si alguien (vos en el futuro, un colaborador) quiere
> entender por qué hay un crate o una herramienta acá, este es el documento.

## Stack en una imagen

```
┌─────────────────────────────────────────────────────────────┐
│ Tauri 2.x  — empaqueta y abre la ventana                    │
│                                                              │
│  ┌──────────────────────┐         ┌─────────────────────┐   │
│  │ Backend Rust         │◄────────┤ Frontend webview    │   │
│  │ • lib.rs (entry)     │ invoke()│ • index.html        │   │
│  │ • daemon.rs (axum)   │         │ • main.js (vanilla) │   │
│  │ • peers.rs           │         │ • style.css         │   │
│  │ • config.rs          │         └─────────────────────┘   │
│  └──────────────────────┘                                   │
│      │                                                       │
│      └─► tokio runtime (daemon HTTP en background)          │
└─────────────────────────────────────────────────────────────┘
```

## Backend — Rust

### Toolchain

| Pieza | Versión | Cómo se instala |
|---|---|---|
| **Rust stable** | 1.96.0 | `winget install Rustlang.Rustup` → `rustup default stable` |
| **Cargo** | 1.96.0 | viene con rustup |
| **MSVC Build Tools 2022** | C++ workload + Windows 11 SDK | `winget install Microsoft.VisualStudio.2022.BuildTools --override "..."` |
| **Tauri CLI v2** | 2.11.2 | `cargo install tauri-cli --locked --version "^2"` |

> En Windows, `cargo build` necesita MSVC para enlazar. La instalación son ~3 GB.
> En Linux/Mac no hace falta (gcc/clang de la distro son suficientes).

### Crates principales (`Cargo.toml`)

| Crate | Versión | Por qué |
|---|---|---|
| **`tauri`** | 2.11.2 | El framework. Abre la webview, expone `invoke()`, empaqueta. |
| **`tauri-build`** | 2.6.2 (build-dep) | Genera el `tauri::generate_context!()` del proyecto. |
| **`tauri-plugin-log`** | 2 | Logueo unificado en debug. Se activa solo si `debug_assertions`. |
| **`tokio`** | 1 (full) | Runtime async. El daemon es totalmente tokio. `full` trae todo lo de I/O. |
| **`axum`** | 0.8 (`multipart`) | Servidor HTTP. Diseñado encima de tokio, ergonomía clara con handlers async. |
| **`tower`** | 0.5 | Middleware compatible con axum. Hoy no se usa explícito; reservado para v0.2. |
| **`reqwest`** | 0.12 (`multipart`, `json`, `rustls-tls`, sin defaults) | Cliente HTTP para hablar con los demás peers. **rustls** y no `native-tls`: cero dependencia de OpenSSL del sistema (importante para CI Linux limpio). |
| **`serde` + `serde_json`** | 1 | Serialización de los JSON del daemon + estructuras compartidas con el frontend. |
| **`dotenvy`** | 0.15 | Lee `.env` al arrancar. No falla si no existe. |
| **`dirs`** | 5 | Cross-platform `data_dir()` para inbox + peers.toml. |
| **`toml`** | 0.8 | Parsea `peers.toml`. |
| **`anyhow`** | 1 | Errors ad-hoc en lugares "tirar para arriba". |
| **`thiserror`** | 2 | Errors tipados que se convierten a HTTP status (ver `DaemonError`). |

### Dev-dependencies

| Crate | Por qué |
|---|---|
| **`tempfile`** | Crear directorios temporales en tests del daemon. |

> `reqwest` también se usa en tests pero ya está en deps principales (peers.rs lo
> necesita en runtime), así que no se duplica.

### Decisiones técnicas del backend

- **`axum` vs `actix-web` vs `warp`**: axum gana por ergonomía moderna + alineación
  con tokio + extractors tipados. Para un MVP es lo más legible.
- **Spawn del daemon en `lib.rs::run()`**: el runtime de tokio se crea ANTES de
  Tauri y se mantiene vivo con `rt.enter()`. Si el daemon falla al bindear,
  se loguea pero no aborta la app — preferimos UI rota con mensaje claro a crash.
- **Errores con `thiserror`**: `DaemonError` impl `IntoResponse` y mapea variantes
  a HTTP status. Si querés agregar un error nuevo, agregás variante + `match`.
- **Sin `tracing`** en MVP: usamos `log::*` + `tauri-plugin-log`. `tracing` sería
  útil con instrumentación pero es overkill para una app personal de tres botones.

## Frontend — HTML/CSS/JS vanilla

### Por qué sin framework

- El frontend es **una sola pantalla con tres cards**. Un framework agregaría más
  superficie (bundler, dependencias) que código.
- Aprende lo "directo": cómo el JS se conecta al backend, sin React/Vue en el medio.
- Sin paso de bundling → `cargo tauri dev` y `cargo tauri build` no necesitan Node.
  Solo Tauri sirve los archivos estáticos desde `src/`.

### Cómo se comunica con Rust

```js
const { invoke } = window.__TAURI__.core;
const peers = await invoke('check_peers');
```

- `window.__TAURI__` lo expone Tauri con `withGlobalTauri: true` en `tauri.conf.json`.
- Sin esto, habría que importar el módulo `@tauri-apps/api` desde npm, lo que
  obligaría a tener un bundler.

### CSP (Content Security Policy)

`tauri.conf.json` → `app.security.csp: null` en MVP. Esto significa "sin CSP
estricto". Está OK porque la UI no hace `fetch` directo a hosts externos — toda
la red la hace el backend Rust. Cuando quede mejor revisar para v0.2.

## Empaquetado — Tauri

### Qué genera `cargo tauri build` en Windows

1. **`target/release/jam-drop.exe`** — binario standalone (~4.7 MB con LTO + strip).
2. **`target/release/bundle/msi/jam-drop_0.1.0_x64_en-US.msi`** — instalador WiX (2.5 MB).
3. **`target/release/bundle/nsis/jam-drop_0.1.0_x64-setup.exe`** — instalador NSIS (1.9 MB).

### Perfil release optimizado (`Cargo.toml`)

```toml
[profile.release]
panic = "abort"     # sin unwinding → binarios más chicos
codegen-units = 1   # un solo codegen-unit → mejor LTO
lto = true          # link-time optimization
opt-level = "s"     # optimizar por tamaño, no velocidad pura
strip = true        # quitar symbols
```

> El binario de 4.7 MB es **30 veces más chico que la app Electron equivalente**.

### Multi-OS via `tauri-action` en CI

El workflow `.github/workflows/release.yml` corre `tauri-apps/tauri-action@v0`
en una matriz `{windows, macos, ubuntu-22.04}` cuando se empuja un tag `v*`.
Cada runner compila para su OS y la action publica los binarios al Release.

## CI/CD — GitHub Actions

### `.github/workflows/ci.yml`

- **Job `gitleaks`**: corre `gitleaks/gitleaks-action@v2` con `.gitleaks.toml`.
- **Job `rust-checks`**: `cargo fmt --check` + `cargo clippy -D warnings` + `cargo test`.
  Solo si existe `src-tauri/Cargo.toml` (guard por compatibilidad con commits viejos).

### `.github/workflows/release.yml`

- Trigger: push de tag `v*`.
- Matriz: Win + macOS + Linux.
- Acción: `tauri-apps/tauri-action@v0` con `releaseDraft: false` (publica directo).

### Pre-commit hook (`.githooks/pre-commit`)

```
1. gitleaks protect --staged
2. (si hay cambios en src-tauri/) cargo fmt --check + cargo clippy
3. ✓ OK o ✗ aborta el commit
```

Activación tras `git clone`:
```bash
git config core.hooksPath .githooks
```

## Seguridad — gitleaks

Configurado con allowlist mínima en `.gitleaks.toml`:

- Paths ignorados: `.env.example`, `peers.example.toml`, `Cargo.lock`, `package-lock.json`.
- Regex ignorados: `JAMDROP_SHARED_TOKEN=` vacío o con placeholder `<...>`.

> **Nunca** agregar `.env` a la allowlist. Si gitleaks lo flaggea es porque alguien
> está intentando commitearlo. El `.gitignore` debería atajar primero.

## Versiones de plataformas que probamos

| OS | Versión | Estado |
|---|---|---|
| Windows | 11 (24H2) | ✅ Build + ejecución + instalador OK |
| Linux (Ubuntu) | 24.04 LTS (CI) | ✅ Build + tests OK |
| Linux (Ubuntu) | 22.04 LTS (Release CI) | ⏳ A confirmar tras v0.1.0 |
| macOS | latest (CI) | ⏳ A confirmar tras v0.1.0 |
