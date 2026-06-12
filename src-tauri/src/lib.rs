// Punto de entrada de la lib. `main.rs` solo llama a `run()`.
// Aquí montamos el runtime de tokio para el daemon y arrancamos Tauri.

pub mod config;
pub mod daemon;
pub mod peers;

use config::Config;
use peers::{Peer, PeerStatus};
use std::sync::Arc;
use tauri::State;

/// Estado compartido entre todos los comandos. Se inyecta con .manage() en setup.
struct AppState {
    cfg: Config,
}

// ============================================================
// Comandos Tauri — el JS los invoca con invoke('nombre', { ... })
// ============================================================

#[tauri::command]
async fn list_peers() -> Result<Vec<Peer>, String> {
    peers::load_peers().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_peers() -> Result<Vec<PeerStatus>, String> {
    let peers = peers::load_peers().await.map_err(|e| e.to_string())?;
    Ok(peers::check_peers(peers).await)
}

#[tauri::command]
async fn send_file(
    state: State<'_, Arc<AppState>>,
    peer_name: String,
    filename: String,
    data: Vec<u8>,
) -> Result<String, String> {
    let all = peers::load_peers().await.map_err(|e| e.to_string())?;
    let peer = all
        .into_iter()
        .find(|p| p.name == peer_name)
        .ok_or_else(|| format!("peer '{peer_name}' no esta en peers.toml"))?;
    peers::send_file_to(&peer, &state.cfg.shared_token, &filename, data)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn me(state: State<'_, Arc<AppState>>) -> serde_json::Value {
    serde_json::json!({
        "device_name": state.cfg.device_name,
        "bind_ip": state.cfg.bind_ip.to_string(),
        "port": state.cfg.port,
        "inbox": state.cfg.inbox.to_string_lossy(),
        "has_token": !state.cfg.shared_token.is_empty(),
    })
}

#[tauri::command]
fn peers_file_path() -> String {
    peers::peers_path().to_string_lossy().to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cfg = Config::from_env();

    // Runtime de tokio compartido por el daemon y los comandos async.
    let rt = tokio::runtime::Runtime::new().expect("no se pudo crear runtime tokio");

    let cfg_for_daemon = cfg.clone();
    rt.spawn(async move {
        if let Err(e) = daemon::serve(cfg_for_daemon).await {
            log::error!("daemon murió: {e:?}");
        }
    });

    let _guard = rt.enter();
    let state = Arc::new(AppState { cfg: cfg.clone() });

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            list_peers,
            check_peers,
            send_file,
            me,
            peers_file_path
        ])
        .setup(move |app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            log::info!(
                "jam-drop iniciado como '{}' (escuchando en {}:{})",
                cfg.device_name,
                cfg.bind_ip,
                cfg.port
            );
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
