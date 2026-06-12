// Gestión de peers conocidos: lectura de un TOML local + ping concurrente
// para saber cuáles están vivos.
//
// Archivo: <data_dir>/jam-drop/peers.toml
// Formato:
//
//   # peers.toml — agrega un bloque [[peer]] por cada equipo.
//   [[peer]]
//   name = "pc"
//   ip = "10.10.0.2"
//   port = 7777   # opcional, default 7777
//
//   [[peer]]
//   name = "laptop"
//   ip = "10.10.0.4"
//
// Si el archivo no existe, lo creamos con un placeholder al primer arranque
// para que el usuario tenga algo que editar (no falla la app).

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::fs;

const PING_TIMEOUT_MS: u64 = 1500;
const DEFAULT_PORT: u16 = 7777;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub name: String,
    pub ip: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

#[derive(Debug, Clone, Serialize)]
pub struct PeerStatus {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub alive: bool,
    /// Latencia de ida y vuelta en ms si está vivo, `None` si no respondió.
    pub latency_ms: Option<u64>,
    /// Nombre que el peer reportó (puede diferir del configurado).
    pub reported_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PeersFile {
    #[serde(default)]
    peer: Vec<Peer>,
}

/// Ruta del archivo de peers para este usuario.
pub fn peers_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("jam-drop")
        .join("peers.toml")
}

/// Carga peers desde el TOML. Si el archivo no existe, lo crea con un
/// placeholder comentado y devuelve lista vacía.
pub async fn load_peers() -> anyhow::Result<Vec<Peer>> {
    let path = peers_path();
    if !fs::try_exists(&path).await.unwrap_or(false) {
        ensure_placeholder(&path).await?;
        return Ok(vec![]);
    }
    let text = fs::read_to_string(&path).await?;
    let parsed: PeersFile = toml::from_str(&text)?;
    Ok(parsed.peer)
}

async fn ensure_placeholder(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    let placeholder = r#"# jam-drop — lista de peers conocidos.
# Cada bloque [[peer]] es un equipo de tu malla.
#
# Ejemplo (descomenta y completa):
#
# [[peer]]
# name = "pc"
# ip = "10.10.0.2"
# port = 7777      # opcional, default 7777
#
# [[peer]]
# name = "laptop"
# ip = "10.10.0.4"
"#;
    fs::write(path, placeholder).await?;
    Ok(())
}

/// Pingea a cada peer en paralelo y devuelve su estado. No falla si alguno
/// no responde — solo lo marca como `alive=false`.
pub async fn check_peers(peers: Vec<Peer>) -> Vec<PeerStatus> {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(PING_TIMEOUT_MS))
        .build()
    {
        Ok(c) => c,
        Err(_) => return peers.into_iter().map(peer_status_offline).collect(),
    };

    let mut tasks = Vec::with_capacity(peers.len());
    for p in peers {
        let client = client.clone();
        tasks.push(tokio::spawn(async move { ping_one(&client, p).await }));
    }
    let mut out = Vec::with_capacity(tasks.len());
    for t in tasks {
        match t.await {
            Ok(s) => out.push(s),
            Err(_) => continue, // join error: lo ignoramos
        }
    }
    out
}

async fn ping_one(client: &reqwest::Client, peer: Peer) -> PeerStatus {
    let url = format!("http://{}:{}/ping", peer.ip, peer.port);
    let start = std::time::Instant::now();
    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let latency = start.elapsed().as_millis() as u64;
            let reported = resp
                .json::<crate::daemon::PingResponse>()
                .await
                .ok()
                .map(|p| p.name);
            PeerStatus {
                name: peer.name.clone(),
                ip: peer.ip.clone(),
                port: peer.port,
                alive: true,
                latency_ms: Some(latency),
                reported_name: reported,
            }
        }
        _ => peer_status_offline(peer),
    }
}

fn peer_status_offline(peer: Peer) -> PeerStatus {
    PeerStatus {
        name: peer.name,
        ip: peer.ip,
        port: peer.port,
        alive: false,
        latency_ms: None,
        reported_name: None,
    }
}

/// Envia `data` al peer con el token compartido. Devuelve el nombre con que
/// quedó guardado en la inbox del receptor.
pub async fn send_file_to(
    peer: &Peer,
    token: &str,
    filename: &str,
    data: Vec<u8>,
) -> anyhow::Result<String> {
    if token.is_empty() {
        anyhow::bail!("JAMDROP_SHARED_TOKEN está vacío — configurá un token en .env");
    }
    let url = format!("http://{}:{}/upload", peer.ip, peer.port);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;
    let part = reqwest::multipart::Part::bytes(data).file_name(filename.to_string());
    let form = reqwest::multipart::Form::new().part("file", part);

    let resp = client
        .post(&url)
        .header("X-Token", token)
        .multipart(form)
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("upload fallido ({}): {}", status, body);
    }
    let saved: crate::daemon::UploadResponse = resp.json().await?;
    Ok(saved.saved_as)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsea_toml_basico() {
        let toml = r#"
            [[peer]]
            name = "pc"
            ip = "10.10.0.2"

            [[peer]]
            name = "laptop"
            ip = "10.10.0.4"
            port = 9999
        "#;
        let parsed: PeersFile = toml::from_str(toml).unwrap();
        assert_eq!(parsed.peer.len(), 2);
        assert_eq!(parsed.peer[0].name, "pc");
        assert_eq!(parsed.peer[0].port, DEFAULT_PORT);
        assert_eq!(parsed.peer[1].port, 9999);
    }

    #[test]
    fn toml_vacio_no_falla() {
        let parsed: PeersFile = toml::from_str("").unwrap();
        assert!(parsed.peer.is_empty());
    }
}
