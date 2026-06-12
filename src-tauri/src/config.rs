// Configuración del daemon. Lee variables de entorno (cargadas desde .env
// con dotenvy) y aplica defaults razonables.
//
// El `.env` no se commitea (gitignored). Plantilla: `.env.example` en la raíz.

use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    /// Nombre amigable del equipo (lo que ven los demás peers).
    pub device_name: String,
    /// IP donde escucha el daemon. Default: 127.0.0.1 (solo loopback) hasta que
    /// el usuario configure su IP de malla (ej. 10.10.0.2).
    pub bind_ip: IpAddr,
    /// Puerto TCP del daemon.
    pub port: u16,
    /// Token compartido que valida los uploads entrantes.
    /// Si está vacío, los uploads se rechazan (modo seguro por defecto).
    pub shared_token: String,
    /// Carpeta donde se guardan los archivos recibidos.
    pub inbox: PathBuf,
}

impl Config {
    /// Lee la config desde el entorno. No falla si faltan variables;
    /// usa defaults seguros y deja que el caller decida qué hacer.
    pub fn from_env() -> Self {
        // No reportamos error si no hay .env — es opcional para dev/prod.
        let _ = dotenvy::dotenv();

        let device_name =
            std::env::var("JAMDROP_DEVICE_NAME").unwrap_or_else(|_| hostname_fallback());

        let bind_ip = std::env::var("JAMDROP_BIND_IP")
            .ok()
            .and_then(|s| s.parse::<IpAddr>().ok())
            .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));

        let port = std::env::var("JAMDROP_PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(7777);

        let shared_token = std::env::var("JAMDROP_SHARED_TOKEN").unwrap_or_default();

        let inbox = std::env::var("JAMDROP_INBOX")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_inbox());

        Self {
            device_name,
            bind_ip,
            port,
            shared_token,
            inbox,
        }
    }
}

/// Nombre de fallback si JAMDROP_DEVICE_NAME no está seteado.
/// No es crítico — solo cosmética en la UI de los demás peers.
fn hostname_fallback() -> String {
    std::env::var("COMPUTERNAME") // Windows
        .or_else(|_| std::env::var("HOSTNAME")) // unix
        .unwrap_or_else(|_| "desconocido".to_string())
}

/// Inbox por defecto: data dir del usuario + "/jam-drop/inbox".
/// En Windows: %APPDATA%\jam-drop\inbox\
/// En Linux: ~/.local/share/jam-drop/inbox/
/// En Mac:   ~/Library/Application Support/jam-drop/inbox/
fn default_inbox() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("jam-drop")
        .join("inbox")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_son_seguros() {
        // Limpiar entorno para no contaminar
        for var in [
            "JAMDROP_DEVICE_NAME",
            "JAMDROP_BIND_IP",
            "JAMDROP_PORT",
            "JAMDROP_SHARED_TOKEN",
            "JAMDROP_INBOX",
        ] {
            // SAFETY: en tests aceptamos mutar env. Es de un solo hilo en este test.
            unsafe {
                std::env::remove_var(var);
            }
        }
        let cfg = Config::from_env();
        assert_eq!(cfg.bind_ip, IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(cfg.port, 7777);
        assert!(
            cfg.shared_token.is_empty(),
            "token vacío por default = modo seguro"
        );
        assert!(cfg.inbox.ends_with("inbox"));
    }
}
