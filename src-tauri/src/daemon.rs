// Daemon HTTP del MVP. Dos endpoints:
//
//   GET  /ping    — sin auth. Devuelve nombre del equipo + versión.
//                   Lo usan los demás peers para saber si estamos vivos.
//
//   POST /upload  — requiere header `X-Token: <shared_token>`.
//                   Recibe multipart con un campo `file` y lo guarda en
//                   `<inbox>/<nombre_seguro>`. Si el nombre se repite, agrega
//                   un sufijo numérico para no pisar (file (1).txt, etc.).
//
// Decisiones MVP:
// - Sin TLS — confiamos en la malla WireGuard (cifrado punta a punta abajo).
// - Auth simple por token compartido. mTLS o llaves por peer = v2.
// - Sanitización mínima del nombre del archivo: solo basename, sin paths.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::{
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::config::Config;

/// Versión del protocolo del daemon. Si cambiamos endpoints, subir.
pub const PROTOCOL_VERSION: &str = "0.1.0";

/// Estado compartido entre handlers.
#[derive(Debug, Clone)]
pub struct DaemonState {
    pub device_name: String,
    pub shared_token: String,
    pub inbox: PathBuf,
}

impl DaemonState {
    pub fn from_config(cfg: &Config) -> Self {
        Self {
            device_name: cfg.device_name.clone(),
            shared_token: cfg.shared_token.clone(),
            inbox: cfg.inbox.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PingResponse {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    pub saved_as: String,
    pub bytes: u64,
}

/// Errores del daemon que se traducen a respuestas HTTP.
#[derive(Debug, thiserror::Error)]
pub enum DaemonError {
    #[error("falta el header X-Token o no coincide")]
    Unauthorized,
    #[error("token compartido no configurado (servidor en modo seguro: rechaza uploads)")]
    NotConfigured,
    #[error("multipart sin campo `file`")]
    MissingFile,
    #[error("nombre de archivo inválido")]
    BadFilename,
    #[error("error de IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("multipart roto: {0}")]
    Multipart(#[from] axum::extract::multipart::MultipartError),
}

impl IntoResponse for DaemonError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::Unauthorized | Self::NotConfigured => StatusCode::UNAUTHORIZED,
            Self::MissingFile | Self::BadFilename | Self::Multipart(_) => StatusCode::BAD_REQUEST,
            Self::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

/// Construye el router del daemon. Separado del `serve` para que los tests
/// puedan montar el mismo router sin abrir socket real si quisieran.
pub fn router(state: DaemonState) -> Router {
    let state = Arc::new(state);
    Router::new()
        .route("/ping", get(ping_handler))
        .route("/upload", post(upload_handler))
        .with_state(state)
}

/// Arranca el daemon en `cfg.bind_ip:cfg.port`. Bloquea el task hasta que
/// el socket muera. Se invoca dentro de `tokio::spawn` desde lib.rs.
pub async fn serve(cfg: Config) -> anyhow::Result<()> {
    // Asegurar que la inbox existe antes de aceptar tráfico.
    fs::create_dir_all(&cfg.inbox).await?;

    let addr = SocketAddr::new(cfg.bind_ip, cfg.port);
    let state = DaemonState::from_config(&cfg);
    let app = router(state);

    log::info!(
        "daemon escuchando en http://{addr} (inbox: {})",
        cfg.inbox.display()
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// ---------- Handlers ----------

async fn ping_handler(State(state): State<Arc<DaemonState>>) -> Json<PingResponse> {
    Json(PingResponse {
        name: state.device_name.clone(),
        version: PROTOCOL_VERSION.to_string(),
    })
}

async fn upload_handler(
    State(state): State<Arc<DaemonState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, DaemonError> {
    // Modo seguro: si no hay token configurado, NUNCA aceptamos uploads.
    if state.shared_token.is_empty() {
        return Err(DaemonError::NotConfigured);
    }
    // Auth: header X-Token debe coincidir.
    let provided = headers
        .get("X-Token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if provided != state.shared_token {
        return Err(DaemonError::Unauthorized);
    }

    while let Some(field) = multipart.next_field().await? {
        if field.name() != Some("file") {
            continue;
        }
        let filename = field
            .file_name()
            .map(|s| s.to_string())
            .ok_or(DaemonError::BadFilename)?;
        let safe = safe_filename(&filename).ok_or(DaemonError::BadFilename)?;

        let final_path = pick_unique_path(&state.inbox, &safe).await;
        let bytes = stream_to_file(field, &final_path).await?;
        let saved_as = final_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&safe)
            .to_string();

        log::info!("recibido {} ({} bytes)", saved_as, bytes);
        return Ok(Json(UploadResponse { saved_as, bytes }));
    }

    Err(DaemonError::MissingFile)
}

// ---------- helpers ----------

/// Devuelve `Some(basename)` si el nombre es seguro (sin separadores de path
/// ni `..`). Devuelve `None` si está vacío o intenta escapar.
pub fn safe_filename(name: &str) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Solo el basename: descartamos cualquier path que venga.
    let base = Path::new(trimmed).file_name().and_then(|n| n.to_str())?;
    if base == "." || base == ".." || base.is_empty() {
        return None;
    }
    Some(base.to_string())
}

/// Si `<dir>/<name>` existe, prueba `<dir>/<stem> (1).<ext>`, (2), etc.
async fn pick_unique_path(dir: &Path, name: &str) -> PathBuf {
    let candidate = dir.join(name);
    if !fs::try_exists(&candidate).await.unwrap_or(false) {
        return candidate;
    }
    let (stem, ext) = split_stem_ext(name);
    for i in 1..1000 {
        let alt = match &ext {
            Some(e) => format!("{stem} ({i}).{e}"),
            None => format!("{stem} ({i})"),
        };
        let p = dir.join(&alt);
        if !fs::try_exists(&p).await.unwrap_or(false) {
            return p;
        }
    }
    // Fallback raro: tirar con timestamp seria mejor; por ahora pisamos.
    candidate
}

fn split_stem_ext(name: &str) -> (String, Option<String>) {
    let p = Path::new(name);
    let stem = p
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(name)
        .to_string();
    let ext = p
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());
    (stem, ext)
}

async fn stream_to_file(
    mut field: axum::extract::multipart::Field<'_>,
    path: &Path,
) -> Result<u64, DaemonError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    let mut file = fs::File::create(path).await?;
    let mut total: u64 = 0;
    while let Some(chunk) = field.chunk().await? {
        total += chunk.len() as u64;
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    Ok(total)
}

// ============================================================
// Tests
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_filename_acepta_basico() {
        assert_eq!(safe_filename("foto.png"), Some("foto.png".into()));
        assert_eq!(
            safe_filename("informe final.pdf"),
            Some("informe final.pdf".into())
        );
    }

    #[test]
    fn safe_filename_descarta_rutas() {
        assert_eq!(safe_filename("../../etc/passwd"), Some("passwd".into()));
        assert_eq!(
            safe_filename(r"C:\Windows\evil.exe"),
            Some("evil.exe".into())
        );
        assert_eq!(safe_filename("/etc/shadow"), Some("shadow".into()));
    }

    #[test]
    fn safe_filename_rechaza_vacio() {
        assert_eq!(safe_filename(""), None);
        assert_eq!(safe_filename("   "), None);
        assert_eq!(safe_filename("."), None);
        assert_eq!(safe_filename(".."), None);
    }

    #[test]
    fn split_stem_ext_funciona() {
        assert_eq!(
            split_stem_ext("foto.png"),
            ("foto".into(), Some("png".into()))
        );
        assert_eq!(split_stem_ext("README"), ("README".into(), None));
        assert_eq!(
            split_stem_ext("archivo.tar.gz"),
            ("archivo.tar".into(), Some("gz".into()))
        );
    }

    #[tokio::test]
    async fn pick_unique_path_evita_pisar() {
        let tmp = tempfile::tempdir().unwrap();
        // El primero no existe
        let p1 = pick_unique_path(tmp.path(), "test.txt").await;
        assert_eq!(p1, tmp.path().join("test.txt"));
        // Lo creamos
        tokio::fs::write(&p1, b"hola").await.unwrap();
        // El segundo debería ser "test (1).txt"
        let p2 = pick_unique_path(tmp.path(), "test.txt").await;
        assert_eq!(p2, tmp.path().join("test (1).txt"));
    }

    #[tokio::test]
    async fn ping_no_requiere_auth() {
        let tmp = tempfile::tempdir().unwrap();
        let state = DaemonState {
            device_name: "test-host".into(),
            shared_token: "secreto".into(),
            inbox: tmp.path().to_path_buf(),
        };
        let app = router(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let res = reqwest::get(format!("http://{addr}/ping")).await.unwrap();
        assert_eq!(res.status(), 200);
        let body: PingResponse = res.json().await.unwrap();
        assert_eq!(body.name, "test-host");
        assert_eq!(body.version, PROTOCOL_VERSION);
    }

    #[tokio::test]
    async fn upload_sin_token_es_rechazado() {
        let tmp = tempfile::tempdir().unwrap();
        let state = DaemonState {
            device_name: "test-host".into(),
            shared_token: "secreto".into(),
            inbox: tmp.path().to_path_buf(),
        };
        let app = router(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(b"hola".to_vec()).file_name("a.txt"),
        );
        let res = reqwest::Client::new()
            .post(format!("http://{addr}/upload"))
            .multipart(form)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 401);
    }

    #[tokio::test]
    async fn upload_con_token_correcto_guarda_archivo() {
        let tmp = tempfile::tempdir().unwrap();
        let state = DaemonState {
            device_name: "test-host".into(),
            shared_token: "secreto".into(),
            inbox: tmp.path().to_path_buf(),
        };
        let app = router(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let contenido = b"contenido de prueba 1234";
        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(contenido.to_vec()).file_name("doc.txt"),
        );
        let res = reqwest::Client::new()
            .post(format!("http://{addr}/upload"))
            .header("X-Token", "secreto")
            .multipart(form)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body: UploadResponse = res.json().await.unwrap();
        assert_eq!(body.saved_as, "doc.txt");
        assert_eq!(body.bytes, contenido.len() as u64);

        // El archivo está en disco con el contenido correcto.
        let leido = tokio::fs::read(tmp.path().join("doc.txt")).await.unwrap();
        assert_eq!(leido, contenido);
    }

    #[tokio::test]
    async fn upload_sin_token_configurado_devuelve_unauthorized() {
        let tmp = tempfile::tempdir().unwrap();
        let state = DaemonState {
            device_name: "test-host".into(),
            shared_token: "".into(), // sin configurar
            inbox: tmp.path().to_path_buf(),
        };
        let app = router(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(b"x".to_vec()).file_name("a.txt"),
        );
        let res = reqwest::Client::new()
            .post(format!("http://{addr}/upload"))
            .header("X-Token", "cualquiercosa")
            .multipart(form)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 401);
    }
}
