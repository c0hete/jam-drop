# Build y release — jam-drop

> Cómo se compila, cómo se firma cada commit, cómo se publica una versión.

## Comandos diarios

### Dev (con hot-reload)

```bash
cd src-tauri
cargo tauri dev
```

Esto:
1. Compila el backend (incremental).
2. Sirve los archivos estáticos de `src/` como webview.
3. Abre la ventana. Auto-recarga si tocás el frontend.

### Tests

```bash
cd src-tauri
cargo test
```

Cubre: sanitización de nombres, anti path-traversal, ping sin auth, upload con/sin
token, modo seguro cuando el token está vacío, parseo de `peers.toml`, defaults
de config.

**13 tests pasando** (10 daemon + 2 peers + 1 config). En CI Linux: **10 pasan**
(uno es `#[cfg(windows)]` por path separators).

### Lint y formato

```bash
cd src-tauri
cargo fmt --all              # aplica
cargo fmt --all -- --check   # solo verifica
cargo clippy --all-targets -- -D warnings
```

> El pre-commit hook corre fmt + clippy automáticamente si tocás algo en
> `src-tauri/`, así que casi nunca tenés que correrlos a mano.

### Build release

```bash
cd src-tauri
cargo tauri build
```

Tarda 5-10 min (LTO). Genera en `target/release/`:

- `jam-drop.exe` — binario optimizado (~4.7 MB en Win).
- `bundle/msi/jam-drop_0.1.0_x64_en-US.msi` — instalador WiX.
- `bundle/nsis/jam-drop_0.1.0_x64-setup.exe` — instalador NSIS alternativo.

## Pre-commit hook

Ubicación: `.githooks/pre-commit`. Activación una sola vez tras clonar:

```bash
git config core.hooksPath .githooks
```

### Qué corre

1. **`gitleaks`** sobre los archivos staged. Bloquea si detecta secretos.
2. (si hay cambios en `src-tauri/`) **`cargo fmt --all -- --check`** —
   bloquea si el formato no es el esperado.
3. (idem) **`cargo clippy --all-targets -- -D warnings`** —
   bloquea si hay warnings.

### Fallback de PATH

El hook corre con el PATH del shell de git, que puede no tener `~/.cargo/bin`.
El hook detecta esto y agrega `$HOME/.cargo/bin` o
`/c/Users/$USERNAME/.cargo/bin` automáticamente. Para gitleaks tiene un
fallback similar a la ruta de winget.

### Bypass de emergencia

`git commit --no-verify` saltea los hooks. Es una mala señal — preferible
arreglar lo que el hook flaggea.

## CI (`.github/workflows/ci.yml`)

Trigger: push a `main` o cualquier PR.

### Job 1: `gitleaks`

```yaml
- uses: gitleaks/gitleaks-action@v2
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    GITLEAKS_CONFIG: .gitleaks.toml
```

### Job 2: `rust-checks`

Solo corre si existe `src-tauri/Cargo.toml` (guard por commits muy viejos).

1. Setup Rust stable.
2. Instalar deps de WebKit (Linux).
3. Cache de cargo.
4. `cargo fmt --all -- --check`.
5. `cargo clippy --all-targets -- -D warnings`.
6. `cargo test`.

Tiempo típico: **3-4 minutos** sobre Ubuntu.

## Release (`.github/workflows/release.yml`)

Trigger: push de tag `v*` (ej. `v0.1.0`, `v0.2.0-beta`).

### Matriz

- `windows-latest`
- `macos-latest`
- `ubuntu-22.04`

Cada runner compila y la `tauri-apps/tauri-action@v0` sube los artefactos al
GitHub Release.

### Cómo tagear

```bash
git tag -a v0.1.0 -m "primera version usable"
git push origin v0.1.0
```

Eso es todo. En 10-15 min hay un Release en
`https://github.com/c0hete/jam-drop/releases/tag/v0.1.0` con los binarios.

### Configuración del release.yml

```yaml
releaseDraft: false   # publica directo
prerelease: false
projectPath: src-tauri
```

`releaseBody` tiene instrucciones de instalación + config mínima embebidas.

## Artefactos del Release

Por tag, vas a tener:

| Plataforma | Archivo | Tamaño aprox |
|---|---|---|
| Windows | `jam-drop_<v>_x64_en-US.msi` | 2.5 MB |
| Windows | `jam-drop_<v>_x64-setup.exe` (NSIS) | 1.9 MB |
| Linux | `jam-drop_<v>_amd64.AppImage` | ~10 MB |
| Linux | `jam-drop_<v>_amd64.deb` | ~5 MB |
| macOS (Intel) | `jam-drop_<v>_x64.dmg` | ~5 MB |
| macOS (Apple Silicon) | `jam-drop_<v>_aarch64.dmg` | ~5 MB |

## Versionado

Convenciones:
- **`v0.x.y`** — antes del primer "estable" público. La API y el formato
  pueden romper.
- **`v1.0.0`** — primer release "estable". A partir de acá semver estricto.
- **`vX.Y.0-rc.N`** — release candidates si vamos a "estable".

Hoy estamos en **`v0.1.0`** = primer MVP usable.

### Bump de versión

Hay que sincronizar **DOS lugares** + tagear:

1. `src-tauri/Cargo.toml` → `[package].version`
2. `src-tauri/tauri.conf.json` → `"version"`
3. `git commit -am "chore: bump v0.1.1"`
4. `git tag -a v0.1.1 -m "..."`
5. `git push origin main && git push origin v0.1.1`

> Pendiente para v0.2: script `scripts/bump.sh` que tome `0.1.1` y haga
> los reemplazos por nosotros.

## Auto-updater — pendiente

Plan completo en `AVANCE_Y_PENDIENTES.md`. Resumen:

1. `cargo tauri signer generate` → llaves de firma.
2. La privada va a GitHub Secrets como `TAURI_SIGNING_PRIVATE_KEY`.
3. Plugin `tauri-plugin-updater` agregado al `Cargo.toml`.
4. `tauri.conf.json` con `plugins.updater.endpoints` apuntando al feed JSON
   que la action genera al firmar.
5. Handler en `lib.rs::setup()` que chequea update y muestra dialog.
6. `release.yml` actualizado para pasar las env vars de firma.

Sesión dedicada estimada: **1-2 hs**.

## Cómo hago un release manual sin CI

Si CI está caído y necesitás un build urgente:

```bash
cd src-tauri
cargo tauri build
# artefactos en target/release/bundle/
```

Subir manualmente al GitHub Release con `gh release create v0.x.y target/release/bundle/...`.

## Limpieza de builds

```bash
cd src-tauri
cargo clean        # borra target/, libera varios GB
```

Después del próximo `cargo build` se vuelve a poblar (5-10 min la primera vez).

## Caches útiles

- **CI cache**: `Swatinem/rust-cache@v2` está activo en `ci.yml`, cachea por
  `Cargo.lock`. Si cambiás muchas deps, primer run sin cache; los siguientes
  son rápidos.
- **Local**: `~/.cargo/registry` y `target/` son los caches de Rust.
  No los toques.

## Antes de cada release manual

Checklist breve:

- [ ] CI verde sobre `main` (gitleaks + fmt + clippy + test).
- [ ] Versión bumpeada en `Cargo.toml` y `tauri.conf.json`.
- [ ] Probaste `cargo tauri dev` localmente y la ventana abre + ping responde.
- [ ] Probaste `cargo tauri build` localmente.
- [ ] (Si tocaste la doc) actualizaste `documentacion/AVANCE_Y_PENDIENTES.md`.
- [ ] Tag escrito con mensaje significativo (no solo "v0.1.1").
