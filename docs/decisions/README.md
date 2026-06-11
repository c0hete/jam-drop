# Decisiones arquitectónicas (ADRs)

Cada decisión "no obvia" o de impacto duradero queda registrada acá como un ADR
(Architecture Decision Record). Formato corto y vivo.

## Convención

- Archivo: `NNNN-titulo-kebab-case.md` (numeración correlativa).
- Sin borrar ADRs viejos: si una decisión se revierte, se crea un ADR nuevo que
  **supersede** al anterior, y el viejo cambia su estado a `Superado por ADR-XXXX`.

## Estados posibles

- `Propuesto` — bajo discusión.
- `Aceptado` — decidido y vigente.
- `Superado por ADR-XXXX` — reemplazado.
- `Descartado` — propuesto pero no se aplicó (útil para no re-debatirlo).

## Índice

- [0001 — Stack: Tauri 2 + Rust/axum + UI vanilla](0001-stack-tauri-vanilla-axum.md)
