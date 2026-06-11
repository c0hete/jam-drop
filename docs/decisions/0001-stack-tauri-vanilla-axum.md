# ADR-0001 — Stack: Tauri 2 + Rust/axum + UI vanilla

- **Fecha**: 2026-06-11
- **Estado**: Aceptado
- **Decisor**: José

## Contexto

`jam-drop` debe correr en cada uno de los equipos personales del paraguas (PC, laptop,
eventualmente tablet/teléfono) y mover archivos entre ellos a través de la malla
WireGuard. Es un proyecto **personal/portafolio** con dos objetivos paralelos:

1. **Resolver una necesidad real** (compartir archivos entre los propios equipos sin
   nube).
2. **Aprender** un stack moderno que sume al perfil (DevSecOps + Laravel).

Las opciones consideradas fueron:

- **Tauri 2** (Rust backend + webview frontend)
- **Python + FastAPI + Tkinter / web local**
- **Go + Wails** (binario único, UI HTML)
- **Electron** (Node + Chromium)

## Decisión

Adoptamos **Tauri 2.x con backend en Rust + UI vanilla (HTML/CSS/JS sin frameworks)**.
El servidor HTTP del daemon irá en **axum** sobre **tokio**.

## Razones

| Criterio | Por qué Tauri gana |
|---|---|
| Binario chico (~10 MB típico) | Tauri usa la webview del SO; Electron empaqueta Chromium (~100 MB). |
| Aprender lenguaje nuevo de valor | Rust expande el perfil hacia sistemas/networking, ortogonal al Laravel actual. |
| Camino claro a móvil | Tauri 2 soporta Android/iOS; lo dejamos abierto para v2 sin tirar código. |
| UI con HTML/CSS conocido | El front es plano (sin framework en MVP), no exige curva de UI. |
| Seguridad por diseño | Tauri obliga a allowlist de comandos del backend; menos superficie. |
| Empaquetado multi-OS estándar | `tauri-action` en GitHub Actions hace `.msi`, `.AppImage`, `.dmg`. |

## Costos aceptados

- **Curva inicial de Rust** (~1-2 días para sentirse cómodo). Es parte del valor que
  buscamos, no un costo neto.
- En Windows requiere **MSVC Build Tools** (~3 GB de descarga la primera vez).
- Menos tutoriales que Electron — pero la documentación oficial de Tauri es muy buena.

## Alternativas descartadas (y por qué)

- **Python**: era lo más rápido para tener MVP. Lo descartamos porque el objetivo
  paralelo de aprender un lenguaje nuevo pesaba más que la velocidad inicial.
- **Go + Wails**: muy buena opción, equilibrio entre simpleza y poder. Quedó segunda;
  perdió frente a Tauri por dos cosas: (a) Rust suma más al portafolio que Go en
  nuestro contexto, (b) el roadmap móvil de Tauri es más sólido.
- **Electron**: descartado por tamaño del binario y porque no aporta aprendizaje
  nuevo (ya conocemos JS).

## Consecuencias

- Necesitamos MSVC Build Tools en el entorno de desarrollo Windows (consideración
  para futuras máquinas).
- El CI debe instalar deps de WebKit en Linux para builds reproducibles.
- Pre-commit hooks van a incluir `cargo fmt` + `cargo clippy` cuando exista el código.

## Revisión

- A revisar si: la curva inicial de Rust nos frena más de 1 fin de semana sin avanzar
  con valor, o si el flujo de release multi-OS de Tauri da problemas serios.
