pub mod api;
pub mod model;
pub mod settings;
pub mod state;

use crate::model::MetadataEntry;
use crate::settings::Settings;
use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use rusqlite::Connection;
use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};
use zip::ZipArchive;

/// Loads the metadata.db from the ZIP into an in-memory SQLite database.
fn load_database_to_memory(archive: &mut ZipArchive<File>) -> Option<Connection> {
    let mut db_file = archive.by_name("metadata.db").ok()?;

    // 1. Create a temporary file to hold the extracted DB
    let temp_db = tempfile::Builder::new().suffix(".db").tempfile().ok()?;
    let mut output_db = File::create(temp_db.path()).ok()?;
    std::io::copy(&mut db_file, &mut output_db).ok()?;

    // 2. Open physical connection
    let phys_conn = Connection::open(temp_db.path()).ok()?;

    // 3. Create memory connection
    let mut mem_conn = Connection::open_in_memory().ok()?;

    // 4. Use Backup API to copy physical to memory
    {
        let backup = rusqlite::backup::Backup::new(&phys_conn, &mut mem_conn).ok()?;
        backup
            .run_to_completion(5, std::time::Duration::from_millis(1), None)
            .ok()?;
    }

    // println!("Database successfully loaded into memory.");
    // We'll log this in the caller if we can, or just keep it simple.
    // Actually, let's keep it as is for now or move the log inside if we passed the logger.
    // For now, I'll just change the ones in run_server.
    // temp_db is dropped here and deleted from disk automatically
    Some(mem_conn)
}

pub struct RawAppData {
    pub settings: Settings,
    pub metadata: Vec<MetadataEntry>,
    pub db_conn: Option<Connection>,
    pub archive: Option<ZipArchive<File>>,
}

pub fn load_all_data(
    settings_path: Option<PathBuf>,
    log_tx: Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<RawAppData, Box<dyn std::error::Error + Send + Sync>> {
    let log = |msg: String| {
        if let Some(tx) = &log_tx {
            let _ = tx.send(msg.clone());
        }
        println!("{}", msg);
    };

    let log_err = |msg: String| {
        if let Some(tx) = &log_tx {
            let _ = tx.send(format!("ERROR: {}", msg));
        }
        eprintln!("{}", msg);
    };

    log("Lade Konfiguration ...".to_string());

    // 1. Load Settings
    let settings = Settings::new(settings_path)?;

    // 2. Resolve Zip Path
    let mut zip_path = PathBuf::from(&settings.zip_path);

    // If path is relative, resolve it relative to the settings.toml location
    if zip_path.is_relative() {
        if let Some(source_path) = &settings.source_path {
            if let Some(parent) = source_path.parent() {
                zip_path = parent.join(&settings.zip_path);
            }
        }
    }

    log("Lade Archiv ...".to_string());
    println!("Using Message Archive: {:?}", zip_path);

    // 3. Load Metadata and DB
    if !zip_path.exists() {
        log_err(format!("Archiv nicht gefunden unter: {:?}", zip_path));
        return Ok(RawAppData {
            settings,
            metadata: Vec::new(),
            db_conn: None,
            archive: None,
        });
    }

    let file = File::open(&zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Load metadata.json
    let mut metadata = Vec::new();
    match archive.by_name("metadata.json") {
        Ok(mut file) => {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                metadata = serde_json::from_str(&content).unwrap_or_default();
                log(format!("Lade {} Nachrichten ...", metadata.len()));
            }
        }
        Err(e) => log_err(format!("metadata.json nicht gefunden: {}", e)),
    }

    log("Verarbeite Daten ...".to_string());

    // Load database to memory
    let db_conn = load_database_to_memory(&mut archive);
    if db_conn.is_some() {
        log("Datenbank ist bereit.".to_string());
    }

    Ok(RawAppData {
        settings,
        metadata,
        db_conn,
        archive: Some(archive),
    })
}

pub fn init_app_state(
    settings_path: Option<PathBuf>,
    log_tx: Option<tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<AppState, Box<dyn std::error::Error + Send + Sync>> {
    let raw = load_all_data(settings_path, log_tx.clone())?;
    Ok(AppState::new(
        raw.settings,
        raw.metadata,
        raw.db_conn,
        raw.archive,
        log_tx,
    ))
}

pub async fn run_server_with_state(
    app_state: AppState,
    port: u16,
    shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
    log_tx: Option<tokio::sync::mpsc::UnboundedSender<String>>,
    frontend_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let log = |msg: String| {
        if let Some(tx) = &log_tx {
            let _ = tx.send(msg.clone());
        }
        println!("{}", msg);
    };

    let log_err = |msg: String| {
        if let Some(tx) = &log_tx {
            let _ = tx.send(format!("ERROR: {}", msg));
        }
        eprintln!("{}", msg);
    };

    // 5. Setup Router - Robust path resolution for frontend/dist
    let mut dist_path = if let Some(p) = frontend_path {
        p
    } else {
        PathBuf::from("frontend/dist") // Ideal case: CWD is project root
    };

    if !dist_path.exists() {
        let mut possible_paths = vec![
            "../frontend/dist".to_string(),    // CWD is backend/ or launcher/
            "../../frontend/dist".to_string(), // CWD is launcher/src-tauri/
            "frontend/dist".to_string(),       // CWD is root
            "./dist".to_string(),              // if we are inside the dist folder itself (unlikely)
        ];

        // Add path relative to current executable (important for bundled apps)
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // macOS bundle hierarchy: Contents/MacOS/app -> Contents/Resources/frontend/dist
                possible_paths.push(
                    exe_dir
                        .join("../Resources/frontend/dist")
                        .to_string_lossy()
                        .to_string(),
                );
                possible_paths.push(
                    exe_dir
                        .join("../Resources/dist")
                        .to_string_lossy()
                        .to_string(),
                );
                possible_paths.push(exe_dir.join("frontend/dist").to_string_lossy().to_string());
            }
        }

        for p in possible_paths {
            let attempt = PathBuf::from(p);
            if attempt.exists() {
                dist_path = attempt;
                break;
            }
        }
    }

    println!(
        "Frontend path candidate: {:?} (exists: {})",
        dist_path,
        dist_path.exists()
    );

    if !dist_path.exists() {
        log_err("CRITICAL: frontend/dist directory not found! UI will not load.".to_string());
    }

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/labels", get(api::get_labels))
                .route("/query", post(api::search_messages))
                .route("/messages/:id", get(api::get_message))
                .route(
                    "/messages/:id/attachment/:filename",
                    get(api::download_attachment),
                )
                .route("/system/info", get(api::get_system_info))
                .route("/system/select-file", post(api::select_file))
                .route("/system/select-save-file", post(api::select_save_file))
                .route("/system/select-toml", post(api::select_toml_file))
                .route("/system/inspect-toml", post(api::inspect_settings))
                .route("/system/convert", post(api::convert_mbox))
                .route("/system/convert/status", get(api::get_convert_status))
                .route("/system/convert/abort", post(api::abort_convert))
                .route("/system/settings", post(api::update_settings))
                .route("/system/restart", post(api::restart_with_settings)),
        )
        .fallback_service(
            ServeDir::new(dist_path.clone())
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(dist_path.join("index.html"))),
        )
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // 6. Bind to Port
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;
    log(format!("Listening on {}", local_addr));

    // Inject shutdown signal
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    log("Backend server exited gracefully.".to_string());
    Ok(())
}

pub async fn run_server(
    settings_path: Option<PathBuf>,
    port_override: Option<u16>,
    shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
    log_tx: Option<tokio::sync::mpsc::UnboundedSender<String>>,
    frontend_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = init_app_state(settings_path, log_tx.clone())?;
    let port = port_override.unwrap_or(8000);
    run_server_with_state(state, port, shutdown_signal, log_tx, frontend_path).await
}
