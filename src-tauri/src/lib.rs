// Punto de entrada de la lib. `main.rs` solo llama a `run()`.
// Aquí montamos el runtime de tokio para el daemon y arrancamos Tauri.

pub mod config;
pub mod daemon;

use config::Config;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Cargar config (lee .env y defaults).
    let cfg = Config::from_env();

    // Runtime de tokio compartido por el daemon y cualquier task asíncrona.
    let rt = tokio::runtime::Runtime::new().expect("no se pudo crear runtime tokio");

    // Spawneamos el daemon en background. Si falla al bindear, lo logueamos
    // pero seguimos abriendo la UI igual (mejor mostrar la app que crashear).
    let cfg_for_daemon = cfg.clone();
    rt.spawn(async move {
        if let Err(e) = daemon::serve(cfg_for_daemon).await {
            log::error!("daemon murió: {e:?}");
        }
    });

    // Mantener vivo el runtime mientras Tauri corre.
    let _guard = rt.enter();

    tauri::Builder::default()
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
