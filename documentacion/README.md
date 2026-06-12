# Documentación local — jam-drop

> Documentación operativa y técnica del proyecto. Vive **acá** porque crece
> conmigo durante el desarrollo y no es para publicar (a diferencia del
> README.md raíz, que sí es la cara del repo).

## Empezá por acá si retomás el proyecto

📌 **[AVANCE_Y_PENDIENTES.md](AVANCE_Y_PENDIENTES.md)** — snapshot del estado.
Lo realizado por fase, lo pendiente con prioridad, decisiones, gotchas.

## Si querés profundizar

- **[TECNOLOGIAS.md](TECNOLOGIAS.md)** — qué crates y herramientas usamos, versiones, por qué.
- **[ARQUITECTURA.md](ARQUITECTURA.md)** — estructura del código, módulos, flujo end-to-end,
  diagrama de IPs en producción.
- **[CONFIGURACION_Y_OPERACION.md](CONFIGURACION_Y_OPERACION.md)** — `.env`, `peers.toml`,
  inbox, instalación local, diagnóstico, convivencia con WireGuard.
- **[BUILD_Y_RELEASE.md](BUILD_Y_RELEASE.md)** — comandos diarios, pre-commit hook,
  CI/CD, versionado, plan del auto-updater.

## Decisiones formales (ADRs)

Los ADRs (Architecture Decision Records) viven en `../docs/decisions/` (un nivel
arriba). Esos son decisiones grandes y duraderas, formato más estricto.

- `docs/decisions/0001-stack-tauri-vanilla-axum.md` — por qué Tauri 2 + Rust + UI vanilla.

## Convenciones de esta carpeta

- Archivos en **MAYÚSCULAS_CON_GUIONES_BAJOS.md** = documentos vivos del proyecto.
- README.md (este) = índice.
- No commitear screenshots ni archivos pesados acá (van a `documentacion/imagenes/`
  si hace falta, hoy no hay).
- Cualquier doc nueva: agregar entrada acá.
