# Hooks de git para jam-drop

Tras clonar el repo, activá los hooks UNA vez con:

```bash
git config core.hooksPath .githooks
```

Esto le dice a git que use los hooks de esta carpeta en vez de los de `.git/hooks`.
La config queda en `.git/config` (no se versiona, así que cada copia local lo hace una vez).

## Qué hace cada hook

- **`pre-commit`** — corre antes de cada commit:
  - **gitleaks** sobre los archivos staged. Bloquea el commit si detecta secretos.

A futuro (cuando haya código):
- `cargo fmt --check`, `cargo clippy`, `cargo test` para el backend.
- Lint del frontend.

## Bypass de emergencia

`git commit --no-verify` lo saltea — pero es una mala señal. Si tenés que usarlo,
arreglá lo que esté roto en vez de evadirlo.

## Requisitos

- [gitleaks](https://github.com/gitleaks/gitleaks) instalado. En Windows:
  `winget install gitleaks.gitleaks`. En Mac: `brew install gitleaks`.
