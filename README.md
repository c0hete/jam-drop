# jam-drop

> Tu propia app de drag-and-drop de archivos entre equipos personales, sobre tu propia
> red privada. Sin nube ajena.

## Qué es

`jam-drop` corre en cada uno de tus equipos (PC, laptop, eventualmente teléfono).
Cuando querés pasar un archivo de un equipo a otro, abrís la app, elegís el archivo
y el destino, y llega a su inbox. Todo viaja por una red privada que vos administrás
(en mi caso, una [malla WireGuard](https://www.wireguard.com/) propia).

## Estado

🚧 **MVP en desarrollo.** Decisiones arquitectónicas en [docs/decisions/](docs/decisions/).

## Para qué (y para qué NO)

Para:
- Mover archivos puntuales entre tus propios equipos en distintas ubicaciones (casa,
  trabajo, viaje), sin depender de Drive/Dropbox/iCloud.
- Tener una app chiquita propia que funciona sobre tu VPN.

NO para:
- Reemplazar Dropbox para 100 usuarios. No es eso.
- Sincronización continua de carpetas (para eso ya está [Syncthing](https://syncthing.net/)).
- Compartir archivos con otra gente. Es para *vos* y *tus* equipos.

## Stack

- Backend: Rust + axum + tokio
- Frontend: HTML/CSS/JS vanilla
- Empaque: Tauri 2.x → binario chico multi-OS

## Cómo correrlo

(Pendiente — aparece cuando esté el primer binario en GitHub Releases.)

## Licencia

Por definir.
