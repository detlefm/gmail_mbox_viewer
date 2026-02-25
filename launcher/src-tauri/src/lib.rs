use backend::settings::Settings;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::path::BaseDirectory;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, State,
};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

#[derive(serde::Serialize, serde::Deserialize)]
struct LauncherConfig {
    settings_path: String,
    port: u16,
    browser: Option<String>,
}

#[derive(serde::Serialize, Clone)]
struct AppStatus {
    settings_path: String,
    mbxc_path: String,
    port: u16,
    browser: Option<String>,
    status: String,
    error: Option<String>,
    messages: Vec<String>,
}

struct AppState {
    // We hold the shutdown signal sender for the current backend task
    backend_shutdown_tx: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
    current_settings_path: Mutex<String>,
    startup_error: Mutex<Option<String>>,
    mbxc_path: Mutex<String>,
    port: Mutex<u16>,
    browser: Mutex<Option<String>>,
    status: Mutex<String>,
    messages: Mutex<VecDeque<String>>,
    cached_backend_state: Mutex<Option<(Arc<backend::state::AppState>, String)>>,
}

fn save_config(app: &AppHandle, path: &str, port: u16, browser: Option<String>) {
    if let Ok(config_dir) = app.path().app_config_dir() {
        let _ = std::fs::create_dir_all(&config_dir);
        let config_path = config_dir.join("launcher_config.json");
        let config = LauncherConfig {
            settings_path: path.to_string(),
            port,
            browser,
        };
        if let Ok(json) = serde_json::to_string(&config) {
            let _ = std::fs::write(config_path, json);
        }
    }
}

fn load_config(app: &AppHandle) -> Option<(String, u16, Option<String>)> {
    if let Ok(config_dir) = app.path().app_config_dir() {
        let config_path = config_dir.join("launcher_config.json");
        if let Ok(json) = std::fs::read_to_string(config_path) {
            if let Ok(config) = serde_json::from_str::<LauncherConfig>(&json) {
                return Some((config.settings_path, config.port, config.browser));
            }
        }
    }
    None
}

#[tauri::command]
async fn select_settings_file(app: AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .add_filter("Settings", &["toml"])
        .pick_file(move |file_path| {
            let _ = tx.send(file_path);
        });

    match rx.await {
        Ok(Some(p)) => Ok(Some(p.to_string())),
        Ok(None) => Ok(None),
        Err(_) => Err("Failed to receive file selection".to_string()),
    }
}

#[tauri::command]
fn get_current_settings(state: State<AppState>) -> String {
    state.current_settings_path.lock().unwrap().clone()
}

#[tauri::command]
fn get_app_status(state: State<AppState>) -> AppStatus {
    AppStatus {
        settings_path: state.current_settings_path.lock().unwrap().clone(),
        mbxc_path: state.mbxc_path.lock().unwrap().clone(),
        port: *state.port.lock().unwrap(),
        browser: state.browser.lock().unwrap().clone(),
        status: state.status.lock().unwrap().clone(),
        error: state.startup_error.lock().unwrap().clone(),
        messages: state.messages.lock().unwrap().iter().cloned().collect(),
    }
}

#[tauri::command]
async fn update_global_port(
    app: AppHandle,
    state: State<'_, AppState>,
    port: u16,
) -> Result<(), String> {
    // 1. Update state
    {
        let mut port_guard = state.port.lock().unwrap();
        *port_guard = port;
    }

    // 2. Persist
    let settings_path = state.current_settings_path.lock().unwrap().clone();
    let browser = state.browser.lock().unwrap().clone();
    save_config(&app, &settings_path, port, browser);

    // 3. Restart backend with new port (but don't reload data)
    restart_backend(app, state, settings_path, false).await
}

async fn restart_backend(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    force_reload: bool,
) -> Result<(), String> {
    // 1. Stop existing backend
    stop_backend_server(&state);

    // Give OS/Runtime a moment to clean up resources (port, threads)
    std::thread::sleep(std::time::Duration::from_millis(150));

    // 2. Clear error state
    *state.startup_error.lock().unwrap() = None;

    // 3. Update current path and port in state and persist
    {
        let mut path_guard = state.current_settings_path.lock().unwrap();
        *path_guard = path.clone();
        let port = *state.port.lock().unwrap();
        let browser = state.browser.lock().unwrap().clone();
        save_config(&app, &path, port, browser);
    }

    // 4. Start new backend
    match start_backend_server(&app, &path, force_reload) {
        Ok(_) => {
            println!("Backend server task spawned.");
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to start backend: {}", e);
            *state.startup_error.lock().unwrap() = Some(e.to_string());
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn restart_backend_with_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<(), String> {
    restart_backend(app, state, path, true).await
}

#[tauri::command]
async fn update_browser(
    app: AppHandle,
    state: State<'_, AppState>,
    browser: Option<String>,
) -> Result<(), String> {
    {
        let mut browser_guard = state.browser.lock().unwrap();
        *browser_guard = browser.clone();
    }
    let settings_path = state.current_settings_path.lock().unwrap().clone();
    let port = *state.port.lock().unwrap();
    save_config(&app, &settings_path, port, browser);
    Ok(())
}

#[tauri::command]
fn open_frontend(app: AppHandle, state: State<AppState>) {
    let port = *state.port.lock().unwrap();
    let browser = state.browser.lock().unwrap().clone();
    let url = format!("http://localhost:{}", port);
    let _ = app.opener().open_url(url, browser.as_deref());
}

#[tauri::command]
fn quit_app(app: AppHandle, state: State<AppState>) {
    stop_backend_server(&state);
    app.exit(0);
}

#[tauri::command]
async fn convert_mbox(app: AppHandle, input: String, output: String) -> Result<(), String> {
    let input_path = std::path::PathBuf::from(input);
    let output_path = std::path::PathBuf::from(output);

    let app_clone = app.clone();
    
    // We run the conversion in a separate thread to avoid blocking the async runtime or UI
    tokio::task::spawn_blocking(move || {
        let progress_callback = Some(Box::new(move |bytes, count| {
            println!("Backend: Progress {} emails, {} bytes", count, bytes);
            let _ = app_clone.emit("conversion-progress", serde_json::json!({
                "bytes": bytes,
                "count": count
            }));
        }) as Box<dyn Fn(u64, u64) + Send>);

        mbox2zip::convert_mbox_to_mbxc(input_path, output_path, progress_callback)
            .map_err(|e| e.to_string())
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
fn open_converter(app: AppHandle) {
    if let Some(window) = app.get_webview_window("converter") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
async fn select_mbox_file(app: AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .add_filter("MBOX", &["mbox"])
        .pick_file(move |file_path| {
            let _ = tx.send(file_path);
        });

    match rx.await {
        Ok(Some(p)) => Ok(Some(p.to_string())),
        Ok(None) => Ok(None),
        Err(_) => Err("Failed to receive file selection".to_string()),
    }
}

#[tauri::command]
async fn select_mbxc_file(app: AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .add_filter("MBXC", &["mbxc"])
        .save_file(move |file_path| {
            let _ = tx.send(file_path);
        });

    match rx.await {
        Ok(Some(p)) => Ok(Some(p.to_string())),
        Ok(None) => Ok(None),
        Err(_) => Err("Failed to receive file selection".to_string()),
    }
}

#[tauri::command]
async fn get_file_size(path: String) -> Result<u64, String> {
    std::fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn hide_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }
}

fn stop_backend_server(state: &AppState) {
    let mut guard = state.backend_shutdown_tx.lock().unwrap();
    if let Some(tx) = guard.take() {
        let _ = tx.send(()); // Send shutdown signal
        println!("Backend server shutdown signal sent.");
    }
    // Reset status on stop
    *state.status.lock().unwrap() = "Stopped".to_string();
}

fn start_backend_server(
    app: &AppHandle,
    settings_path_str: &str,
    force_reload: bool,
) -> std::io::Result<()> {
    let state = app.state::<AppState>();
    let settings_path = PathBuf::from(settings_path_str);

    // Basic validation
    if !settings_path.exists() {
        *state.status.lock().unwrap() = "Error: Settings file not found".to_string();
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {:?}", settings_path),
        ));
    }

    // Parse settings to get MBXC for the UI
    let current_port = *state.port.lock().unwrap();
    let current_browser = state.browser.lock().unwrap().clone();
    if let Ok(settings) = Settings::new(Some(settings_path.clone())) {
        *state.mbxc_path.lock().unwrap() = settings.zip_path.clone();
        let _ = app.emit(
            "backend-config",
            AppStatus {
                settings_path: settings_path_str.to_string(),
                mbxc_path: settings.zip_path,
                port: current_port,
                browser: current_browser,
                status: "Loading...".to_string(),
                error: None,
                messages: vec![],
            },
        );
    }
    *state.status.lock().unwrap() = "Loading...".to_string();
    let _ = app.emit("backend-status", "Loading...");

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    // Store the sender so we can stop it later
    *state.backend_shutdown_tx.lock().unwrap() = Some(tx);

    let (log_tx, mut log_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    // Monitor logs
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app_clone.state::<AppState>();
        while let Some(msg) = log_rx.recv().await {
            {
                let mut messages = state.messages.lock().unwrap();
                messages.push_back(msg.clone());
                if messages.len() > 21 {
                    messages.pop_front();
                }
            }
            let _ = app_clone.emit("backend-log", msg.clone());

            // Check for explicit errors in the log
            if msg.starts_with("ERROR:") {
                let err_detail = msg[6..].trim().to_string();
                *state.status.lock().unwrap() = format!("Error: {}", err_detail);
                let _ = app_clone.emit("backend-status", format!("Error: {}", err_detail));
            }
            // Update status if we see "Listening", but ONLY if we don't already have an error
            else if msg.contains("Listening on") {
                let mut status_guard = state.status.lock().unwrap();
                if !status_guard.starts_with("Error:") {
                    *status_guard = "Running".to_string();
                    let _ = app_clone.emit("backend-status", "Running");

                    // Extract port from "Listening on 127.0.0.1:54321"
                    if let Some(pos) = msg.rfind(':') {
                        if let Ok(p) = msg[pos + 1..].parse::<u16>() {
                            *state.port.lock().unwrap() = p;
                            let browser = state.browser.lock().unwrap().clone();
                            let _ = app_clone.emit(
                                "backend-config",
                                AppStatus {
                                    settings_path: state
                                        .current_settings_path
                                        .lock()
                                        .unwrap()
                                        .clone(),
                                    mbxc_path: state.mbxc_path.lock().unwrap().clone(),
                                    port: p,
                                    browser,
                                    status: "Running".to_string(),
                                    error: None,
                                    messages: state
                                        .messages
                                        .lock()
                                        .unwrap()
                                        .iter()
                                        .cloned()
                                        .collect(),
                                },
                            );
                        }
                    }
                }
            }
        }
    });

    // Spawn the backend server as a Tokio task
    let app_clone_for_backend = app.clone();
    let settings_path_str_owned = settings_path_str.to_string();
    tauri::async_runtime::spawn(async move {
        let state = app_clone_for_backend.state::<AppState>();
        // 1. Get or Load State
        let backend_state = {
            let mut cache = state.cached_backend_state.lock().unwrap();
            let mut use_cache = false;
            if !force_reload {
                if let Some((_, cached_path)) = cache.as_ref() {
                    if cached_path == &settings_path_str_owned {
                        use_cache = true;
                    }
                }
            }

            if use_cache {
                println!("Reusing cached backend state for data.");
                cache.as_ref().unwrap().0.clone()
            } else {
                println!("Loading fresh backend state (force={}).", force_reload);
                match backend::init_app_state(
                    Some(settings_path.clone()),
                    Some(log_tx.clone()),
                ) {
                    Ok(s) => {
                        let arc_s = Arc::new(s);
                        *cache = Some((arc_s.clone(), settings_path_str_owned.clone()));
                        arc_s
                    }
                    Err(e) => {
                        let err_msg = format!("Init Error: {}", e);
                        *state.status.lock().unwrap() = err_msg.clone();
                        let _ = app_clone_for_backend.emit("backend-status", err_msg);
                        return;
                    }
                }
            }
        };

        // 2. Setup shutdown signal
        let shutdown_signal = async {
            rx.await.ok();
            println!("Backend received shutdown signal.");
        };
        // 3. Run Server
        let port = *state.port.lock().unwrap();

        // Resolve frontend path from resources - try multiple common names
        let mut frontend_path = None;
        let candidates = [
            "frontend/dist",
            "dist",
            "_up_/frontend/dist",
            "_up_/_up_/frontend/dist",
            "index.html",
        ];

        for candidate in candidates {
            if let Ok(p) = app_clone_for_backend
                .path()
                .resolve(candidate, BaseDirectory::Resource)
            {
                let p_actual = if candidate == "index.html" {
                    p.parent().unwrap().to_path_buf()
                } else {
                    p
                };
                if p_actual.exists() && p_actual.is_dir() {
                    let msg = format!(
                        "SUCCESS: Found frontend at Resource path: {:?} (via '{}')",
                        p_actual, candidate
                    );
                    println!("{}", msg);
                    frontend_path = Some(p_actual);
                    break;
                } else {
                    let msg = format!("Resource candidate '{}' resolved to {:?}, but it doesn't exist or isn't a dir.", candidate, p_actual);
                    println!("{}", msg);
                }
            }
        }

        if frontend_path.is_none() {
            let msg = "WARNING: Could not resolve frontend/dist via any standard Resource path. Falling back to local search.".to_string();
            println!("{}", msg);
        }

        if let Err(e) = backend::run_server_with_state(
            (*backend_state).clone(),
            port,
            shutdown_signal,
            Some(log_tx),
            frontend_path,
        )
        .await
        {
            let err_msg = format!("Error: {}", e);
            eprintln!("Backend server error: {}", e);
            *state.status.lock().unwrap() = err_msg.clone();
            let _ = app_clone_for_backend.emit("backend-status", err_msg);
        } else {
            println!("Backend server exited gracefully.");
            *state.status.lock().unwrap() = "Stopped".to_string();
            let _ = app_clone_for_backend.emit("backend-status", "Stopped");
        }
    });

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .as_ref()
                .map(|w| {
                    let _ = w.show();
                    let _ = w.set_focus();
                });
        }))
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            backend_shutdown_tx: Mutex::new(None),
            current_settings_path: Mutex::new(String::from("../settings.toml")),
            startup_error: Mutex::new(None),
            mbxc_path: Mutex::new(String::new()),
            port: Mutex::new(8000),
            browser: Mutex::new(None),
            status: Mutex::new(String::from("Stopped")),
            messages: Mutex::new(VecDeque::new()),
            cached_backend_state: Mutex::new(None),
        })
        .setup(|app| {
            // Load persisted settings if they exist
            if let Some((persisted_path, persisted_port, persisted_browser)) =
                load_config(app.app_handle())
            {
                let state = app.state::<AppState>();
                *state.current_settings_path.lock().unwrap() = persisted_path;
                *state.port.lock().unwrap() = persisted_port;
                *state.browser.lock().unwrap() = persisted_browser;
            }

            // Create Native App Menu (primarily for macOS where it's mandatory)
            #[cfg(target_os = "macos")]
            {
                let menu = tauri::menu::Menu::default(app.handle())?;
                app.set_menu(menu)?;
            }

            // Create Tray Menu
            let open_frontend_item =
                MenuItem::with_id(app, "open_frontend", "Open Frontend", true, None::<&str>)?;
            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&open_frontend_item, &settings_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app: &AppHandle, event| match event.id.as_ref() {
                    "open_frontend" => {
                        let state = app.state::<AppState>();
                        let port = *state.port.lock().unwrap();
                        let browser = state.browser.lock().unwrap().clone();
                        let url = format!("http://localhost:{}", port);
                        let _ = app.opener().open_url(url, browser.as_deref());
                    }
                    "settings" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // Start backend on startup
            let state = app.state::<AppState>();
            let settings_path = state.current_settings_path.lock().unwrap().clone();

            // Attempt start
            match start_backend_server(app.handle(), &settings_path, true) {
                Ok(_) => {
                    // Success: Accessory mode (hidden from dock)
                    #[cfg(target_os = "macos")]
                    let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                }
                Err(e) => {
                    eprintln!("Failed to start backend: {}", e);
                    *state.startup_error.lock().unwrap() = Some(e.to_string());

                    // Failure: Show window AND Regular mode (visible in dock)
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    #[cfg(target_os = "macos")]
                    let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    window.hide().unwrap();
                    api.prevent_close();

                    #[cfg(target_os = "macos")]
                    {
                        let _ = window
                            .app_handle()
                            .set_activation_policy(tauri::ActivationPolicy::Accessory);
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_current_settings,
            restart_backend_with_settings,
            open_frontend,
            quit_app,
            select_settings_file,
            hide_window,
            get_app_status,
            update_global_port,
            update_browser,
            convert_mbox,
            open_converter,
            select_mbox_file,
            select_mbxc_file,
            get_file_size
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
