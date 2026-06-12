# Arquitectura — jam-drop

> Cómo está organizado el código y cómo viaja un archivo de un equipo a otro.

## Estructura del proyecto

```
jam-drop/
├── .env                        # config local (gitignored)
├── .env.example                # plantilla
├── .gitleaks.toml              # config de gitleaks con allowlist
├── .gitignore                  # rieles anti-leak: .env, llaves, target/, etc.
├── .mc                         # marcador mente-colmena del paraguas
├── .githooks/
│   ├── README.md               # cómo activar los hooks
│   └── pre-commit              # gitleaks + cargo fmt + cargo clippy
├── .github/workflows/
│   ├── ci.yml                  # checks en cada push/PR
│   └── release.yml             # build multi-OS al tagear v*
├── docs/decisions/             # ADRs (decisiones arquitectónicas formales)
│   ├── README.md
│   └── 0001-stack-tauri-vanilla-axum.md
├── documentacion/              # docs LOCALES (este folder)
│   ├── README.md
│   ├── TECNOLOGIAS.md
│   ├── ARQUITECTURA.md          ← estás acá
│   ├── CONFIGURACION_Y_OPERACION.md
│   ├── BUILD_Y_RELEASE.md
│   └── AVANCE_Y_PENDIENTES.md
├── CLAUDE.md                   # instrucciones del proyecto para Claude
├── README.md                   # README público del repo
├── src/                        # frontend
│   ├── index.html
│   ├── main.js
│   └── style.css
└── src-tauri/                  # backend Rust + config Tauri
    ├── Cargo.toml
    ├── Cargo.lock
    ├── build.rs                # genera tauri::generate_context!
    ├── tauri.conf.json         # config de Tauri (ventana, identifier, etc.)
    ├── capabilities/
    │   └── default.json        # permisos del frontend al backend
    ├── icons/                  # default de Tauri (placeholder)
    └── src/
        ├── main.rs             # entry: invoca lib::run()
        ├── lib.rs              # bootstrap + comandos Tauri
        ├── config.rs           # lee .env + defaults
        ├── daemon.rs           # axum: /ping y /upload
        └── peers.rs            # peers.toml + ping concurrente + envío
```

## Módulos del backend

### `main.rs`

Una sola línea. Solo invoca `jam_drop_lib::run()`. El `#![cfg_attr(...)]` evita que
en Windows release aparezca una consola negra al lado de la ventana.

### `lib.rs`

Punto de entrada real. Hace 4 cosas, en este orden:

1. Carga `Config::from_env()` (lee `.env` + defaults).
2. Crea un runtime de **tokio** y `spawn` el daemon en background. Si el daemon
   falla al bindear (ej. puerto ocupado), se loguea pero la UI igual abre.
3. Registra el `AppState` con `.manage()` y los comandos Tauri con `.invoke_handler()`.
4. Lanza la ventana Tauri.

Comandos expuestos al frontend:

| Comando | Firma | Para qué |
|---|---|---|
| `me()` | `→ JSON` | Identidad: device_name, bind_ip, port, inbox, has_token |
| `list_peers()` | `→ Vec<Peer>` | Lectura cruda de `peers.toml` |
| `check_peers()` | `→ Vec<PeerStatus>` | Ping concurrente a cada peer, con timeout |
| `send_file(peer_name, filename, data)` | `→ String` (nombre guardado) | Envío multipart con auth |
| `peers_file_path()` | `→ String` | Ruta del peers.toml (para el hint en UI) |

### `config.rs`

`Config` con campos `device_name`, `bind_ip`, `port`, `shared_token`, `inbox`.
`Config::from_env()`:

- Llama `dotenvy::dotenv()` (no falla si no hay `.env`).
- Lee cada variable con `unwrap_or` que aplica defaults seguros.
- Defaults clave:
  - `bind_ip = 127.0.0.1` (loopback, no expone nada por accidente).
  - `shared_token = ""` (vacío = rechaza todo upload — modo seguro).
  - `inbox = dirs::data_dir() / "jam-drop" / "inbox"`.

### `daemon.rs`

Router axum con dos rutas:

#### `GET /ping`

- **Sin auth**. Cualquiera en la red puede llamarlo.
- Devuelve `{name, version}`.
- Usado por los demás peers para chequeo vivo/muerto.

#### `POST /upload`

- **Requiere `X-Token` header** que matchee `shared_token`.
- Si `shared_token` está vacío → **siempre 401**, incluso con token correcto
  (modo seguro: el server no acepta nada hasta que el usuario configure token).
- Recibe multipart. Lee el primer field llamado `file`.
- Sanitiza el nombre con `safe_filename()`:
  - Quita paths absolutos / relativos (`Path::new(...).file_name()`).
  - Rechaza vacío, `.`, `..`.
- Auto-renombra si colisiona: `doc.txt` → `doc (1).txt` → `doc (2).txt` …
- Stream-to-file con tokio (no carga todo en memoria).
- Devuelve `{saved_as, bytes}`.

`DaemonError` impl `IntoResponse`:

| Error | HTTP status |
|---|---|
| `Unauthorized` / `NotConfigured` | 401 |
| `MissingFile` / `BadFilename` / `Multipart(_)` | 400 |
| `Io(_)` | 500 |

### `peers.rs`

Maneja la lista de peers conocidos.

- `peers_path()` → `dirs::data_dir() / "jam-drop" / "peers.toml"`.
- `load_peers()` → si el archivo no existe lo crea con placeholder comentado.
- `check_peers()` → para cada peer hace `GET /ping` con timeout de 1.5s,
  todos en paralelo con `tokio::spawn`. Devuelve `PeerStatus` con `alive`,
  `latency_ms`, `reported_name`.
- `send_file_to()` → arma multipart form con `reqwest::multipart`, agrega
  header `X-Token`, hace POST, parsea respuesta `UploadResponse`.

## Frontend (`src/`)

### `index.html`

Estructura simple:
- Header con título + identidad (`soy "..."`).
- Card "Equipos en la malla" con lista de peers + botón refrescar.
- Card "Enviar archivo" con form file + select destino + botón enviar.
- Footer con link al repo.

### `main.js`

Vanilla, ~150 líneas. Funciones:

- `loadMe()` — al arrancar, llama `invoke('me')` y pinta el header.
- `refresh()` — llama `invoke('check_peers')`, reemplaza la lista atómicamente
  con `replaceChildren()`. Lock `refreshing` evita refreshes solapados.
  Indicador `::after` animado durante el escaneo.
- `rebuildDestOptions(alive)` — repuebla el `<select>` con solo peers vivos,
  preserva selección previa si sigue viva.
- Submit del form: lee archivo con `arrayBuffer()`, lo manda como `Array.from(Uint8Array)`
  al comando `send_file`. Muestra ✓ o ✗ según resultado.
- `setInterval(refresh, 10000)` — auto-refresh cada 10s.

`escapeHtml()` aplicado a TODO lo que viene de Rust (nombres, IPs, errores).

### `style.css`

Tema dark con paleta inspirada en Tokyo Night:
- `--bg: #0f1116`
- `--accent: #7aa2f7`
- `--ok: #9ece6a`
- `--error: #f7768e`

Sin frameworks. Cards con `--surface` y border sutil.

## Flujo end-to-end: enviar un archivo PC → laptop

```
[PC, frontend]
  Usuario elige archivo + destino "laptop" + click Enviar
       │
       ▼  invoke('send_file', { peerName: 'laptop', filename, data })
[PC, lib.rs::send_file]
  → busca el peer "laptop" en peers.toml
  → llama peers::send_file_to(&peer, &shared_token, ...)
       │
       ▼  HTTP POST multipart, header X-Token
[red WireGuard: 10.10.0.2 → 10.10.0.4]
       │
       ▼
[Laptop, daemon.rs::upload_handler]
  → valida X-Token contra shared_token (mismo en .env de ambos)
  → lee multipart, sanitiza nombre, auto-renombra si colisiona
  → escribe en %APPDATA%\jam-drop\inbox\<archivo>
  → responde 200 {saved_as, bytes}
       │
       ▼  fluye de vuelta por la misma malla
[PC, frontend]
  Muestra "✓ enviado como <saved_as>"
```

> Todo el tráfico va **encapsulado dentro del túnel WireGuard**. Internet
> pública solo ve UDP cifrado entre las IPs externas de los equipos.

## Concurrencia y runtime

- **Un runtime de tokio**, compartido por:
  - El daemon (`tokio::net::TcpListener` + `axum::serve`).
  - Cada comando Tauri async (cuando el frontend hace `invoke`, Tauri lo despacha
    a un task del runtime).
- **Estado compartido** vía `Arc<AppState>` con `.manage()`. Los handlers Tauri
  reciben `State<'_, Arc<AppState>>` como argumento extra.
- **Sin Mutex** en el MVP — el estado es read-only tras inicialización.

## Por qué el frontend NO usa `fetch` directo

Sería tentador: `fetch('http://10.10.0.4:7777/upload', ...)`. Pero:

1. **Tauri tiene CSP** y por defecto bloquea fetch a hosts arbitrarios.
   Habría que aflojar la security, exponiendo más superficie.
2. **Mantener el contrato cliente↔servidor en un solo lugar** (Rust): el día
   que cambie el protocolo (auth con llaves, mTLS, etc.) solo se toca Rust.
3. **Móvil**: Tauri Mobile usa el mismo `invoke()`. Si el JS hace fetch,
   en Android va a chocar con la network security config.
4. **Concurrencia**: `tokio::spawn` por peer es trivial; en JS habría que
   coordinar con `Promise.all` y los timeouts son más torpes.

## Diagrama de IPs (el escenario real)

```
                  Internet pública
                         │
                  WireGuard túnel cifrado
                         │
        ┌────────────────┴─────────────────┐
        │     Hub: mailcow @ Contabo NY    │
        │     IP malla 10.10.0.1           │
        │     (rutea entre spokes)         │
        └────────────────┬─────────────────┘
                         │
        ┌────────────────┼─────────────────┐
        │                │                 │
   ┌────▼────┐      ┌────▼────┐       ┌────▼────┐
   │   PC    │      │ teléfono│       │ laptop  │
   │10.10.0.2│      │10.10.0.3│       │10.10.0.4│
   │ jam-drop│      │   (sin  │       │ jam-drop│
   │ :7777   │      │ jam-drop)│      │ :7777   │
   └─────────┘      └─────────┘       └─────────┘
```

jam-drop corre en los equipos que tienen capacidad de escuchar en HTTP
(PC, laptop). El teléfono está en la malla pero todavía no corre la app
(eso es para v0.4 cuando hagamos Tauri Mobile).
