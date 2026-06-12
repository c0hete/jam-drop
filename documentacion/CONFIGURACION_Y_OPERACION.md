# Configuración y operación — jam-drop

> Dónde viven los archivos, qué configurar en cada equipo, cómo se opera.

## Variables de entorno (`.env`)

Vive **al lado del binario** o en la raíz del proyecto si corrés desde fuente.
`dotenvy` lo lee desde el directorio de trabajo (CWD) al arrancar.

| Variable | Default | Notas |
|---|---|---|
| `JAMDROP_DEVICE_NAME` | `$COMPUTERNAME` (Win) / `$HOSTNAME` (Unix) | Nombre amigable que ven los demás peers |
| `JAMDROP_BIND_IP` | `127.0.0.1` | IP donde escucha el daemon. En producción usar la IP de la malla |
| `JAMDROP_PORT` | `7777` | Puerto TCP |
| `JAMDROP_SHARED_TOKEN` | (vacío) | Mismo en TODOS los equipos. Vacío = no acepta ningún upload |
| `JAMDROP_INBOX` | `<data_dir>/jam-drop/inbox` | Ruta absoluta opcional |
| `JAMDROP_LOG_LEVEL` | `info` | trace / debug / info / warn / error |

### Generar el token

```bash
openssl rand -hex 32
# o en bash sin openssl:
head -c 32 /dev/urandom | xxd -p -c 64 | tr -d '\n'
```

64 caracteres hex = 256 bits. **El mismo valor va en todos los `.env`** de los
equipos que se quieren hablar.

### Ejemplo (`.env` real, con valores ficticios)

```env
JAMDROP_DEVICE_NAME=pc
JAMDROP_BIND_IP=10.10.0.2
JAMDROP_PORT=7777
JAMDROP_SHARED_TOKEN=9a90f9fc...                 # 64 chars, el mismo en TODOS los equipos
JAMDROP_LOG_LEVEL=info
```

## Lista de peers (`peers.toml`)

Vive en:

| OS | Ruta |
|---|---|
| Windows | `%APPDATA%\jam-drop\peers.toml` (= `C:\Users\<usuario>\AppData\Roaming\jam-drop\peers.toml`) |
| Linux | `~/.local/share/jam-drop/peers.toml` |
| macOS | `~/Library/Application Support/jam-drop/peers.toml` |

Se crea automáticamente con un placeholder comentado al primer arranque
si no existe. Después se edita a mano.

### Formato

```toml
[[peer]]
name = "pc"
ip = "10.10.0.2"
port = 7777        # opcional, default 7777

[[peer]]
name = "laptop"
ip = "10.10.0.4"

[[peer]]
name = "telefono"
ip = "10.10.0.3"
```

> **Tu propio equipo también va en la lista** — es útil para test de humo
> (deberías verte "vivo" a vos mismo si el daemon corre).

## Inbox — dónde llegan los archivos

| OS | Ruta default |
|---|---|
| Windows | `%APPDATA%\jam-drop\inbox\` |
| Linux | `~/.local/share/jam-drop/inbox/` |
| macOS | `~/Library/Application Support/jam-drop/inbox/` |

Configurable con `JAMDROP_INBOX` si querés otra ruta. La carpeta se crea
automáticamente al primer upload.

**Auto-rename si colisiona**: `doc.txt` → `doc (1).txt` → `doc (2).txt` …

## Instalación local (sin .msi)

Útil mientras desarrollás o cuando querés tener el binario "deployable" sin
pasar por un instalador.

Carpeta sugerida en Windows: **`%LOCALAPPDATA%\Programs\jam-drop\`** (estándar
de apps de usuario, no requiere admin).

Archivos que vivirán ahí:

- `jam-drop.exe` ← copiado de `src-tauri/target/release/jam-drop.exe`
- `.env` ← tu config local
- `icon.ico` ← copiado de `src-tauri/icons/icon.ico` (para que el shortcut tenga
  ícono — no es estrictamente necesario)

### Crear el shortcut en el escritorio (PowerShell)

```powershell
$desktop = [Environment]::GetFolderPath('Desktop')
$installDir = Join-Path $env:LOCALAPPDATA 'Programs\jam-drop'
$ws = New-Object -ComObject WScript.Shell
$sc = $ws.CreateShortcut((Join-Path $desktop 'jam-drop.lnk'))
$sc.TargetPath = Join-Path $installDir 'jam-drop.exe'
$sc.WorkingDirectory = $installDir   # CLAVE: para que el .env se lea desde acá
$sc.IconLocation = (Join-Path $installDir 'icon.ico')
$sc.Description = 'jam-drop — drag-and-drop sobre tu malla'
$sc.Save()
```

> **`WorkingDirectory` es crítico**. Si no lo seteás, el shortcut lanza el
> binario desde otro lado y `.env` no se carga.

## Instalación con .msi

1. Doble clic al `.msi`. Acepta UAC.
2. Se instala en `C:\Program Files\jam-drop\`. Crea entrada en menú Inicio.
3. **Copiar `.env`** a `C:\Program Files\jam-drop\.env` (pide UAC para escribir ahí).
4. Lanzá la app desde el menú Inicio.

> Alternativa para no requerir admin para editar `.env`: usar el camino "instalación
> local" descrito arriba en lugar del `.msi`.

## Diagnóstico rápido

| Síntoma | Probable causa | Qué mirar |
|---|---|---|
| Header dice "soy '<COMPUTERNAME>'" en vez del que configuré | `.env` no se cargó | Confirmá que está al lado del .exe **y** que el shortcut tiene `WorkingDirectory` correcto |
| Header dice "⚠ sin token" | `JAMDROP_SHARED_TOKEN` vacío | Editar `.env` |
| Yo mismo aparezco como "○ sin respuesta" | El daemon no bindea o la IP es incorrecta | Mirá los logs (vienen por stdout en dev, en release no hay aún). Confirmá que `10.10.0.X` está en una interfaz local con `ipconfig` (Win) o `ip addr` (Linux) |
| Un peer aparece "○ sin respuesta" pero pingea con `ping <ip>` | La app del peer no corre, o su `JAMDROP_BIND_IP` es `127.0.0.1` | Abrir la app en el peer, confirmar que dice la IP de la malla en el header |
| Envío falla "401 Unauthorized" | Tokens no coinciden | Confirmar `JAMDROP_SHARED_TOKEN` byte a byte entre los `.env` de origen y destino |
| Envío falla "connection refused" | El daemon del peer no responde | App cerrada en el peer, o puerto distinto, o firewall |
| Envío falla con timeout (60s) | Red mala, archivo enorme, o el peer caído mitad-de-envío | Reintentar |

## Logs

En desarrollo (`cargo tauri dev`):

- stdout de la terminal donde lanzaste el dev.
- Plugin de log de Tauri activado en debug, nivel `info`.

En release: hoy NO hay log persistido a archivo. **Pendiente para v0.2.**

## Servicios en background

jam-drop **no corre como servicio** todavía. Cuando cerrás la ventana, el daemon
muere. Para que reciba archivos, la app debe estar abierta.

Workarounds temporales:
- Tener el shortcut en `shell:startup` para autoejecutar al login.
- Usar system tray (v0.2) para que la ventana se "minimice" pero el daemon siga.

## Convivencia con WireGuard

jam-drop usa el túnel pero no lo administra. Requisitos:

- El túnel `malla` debe estar **Up** antes de abrir jam-drop (o reiniciar la app
  después de prender el túnel).
- La IP de la malla del equipo (ej. `10.10.0.2`) debe estar asignada a la
  interfaz `malla` y matchear `JAMDROP_BIND_IP`.

Si cambia la IP de la malla (ej. reasignación), hay que editar el `.env` y
reiniciar jam-drop.

## Firewall

En Windows con `ufw`-like defaults, el firewall de Windows **no necesita reglas
adicionales** — el daemon bindea a la IP de la malla `wg0`, no expone nada al
exterior, y la regla "permitir loopback / interfaces propias" del firewall
default deja pasar.

Si tu firewall es muy restrictivo, abrí TCP 7777 **solo para la interfaz `malla`**.
NO lo abras para todas las interfaces.

## Convivencia con otras instalaciones

El identifier `cl.alvaradomazzei.jamdrop` está fijo en `tauri.conf.json`.
Si reinstalás con .msi sobre una instalación existente, el instalador detecta
la versión vieja y la actualiza limpiamente.

Si querés tener dos instalaciones independientes (ej. test/prod) en el mismo
equipo, cambiar el identifier antes de buildear.
