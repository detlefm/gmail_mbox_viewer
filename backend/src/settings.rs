use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub zip_path: String,
    pub filter_labels: Option<Vec<String>>,
    pub special_labels: Option<Vec<String>>,
    pub browser: Option<String>,
    #[serde(skip)]
    pub source_path: Option<std::path::PathBuf>,
}

impl Settings {
    pub fn new(
        custom_path: Option<std::path::PathBuf>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut content = String::new();
        let mut actual_path = None;

        // 1. Try custom path if provided
        if let Some(mut path) = custom_path {
            // Expand ~ if present
            if path.starts_with("~") {
                if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
                    let path_str = path.to_string_lossy();
                    let expanded = path_str.replacen("~", &home.to_string_lossy(), 1);
                    path = std::path::PathBuf::from(expanded);
                }
            }

            if path.exists() {
                println!("Loading settings from custom path: {:?}", path);
                content = fs::read_to_string(&path)?;
                actual_path = Some(path);
            }
        }

        // 2. Try default locations in CWD
        if actual_path.is_none() {
            let paths = ["settings.toml", "../settings.toml", "data/settings.toml"];
            for p in paths {
                let path = Path::new(p);
                if path.exists() {
                    content = fs::read_to_string(path)?;
                    actual_path = Some(path.to_path_buf());
                    break;
                }
            }
        }

        // 3. Try AppData/Application Support
        if actual_path.is_none() {
            let mut app_data_path = None;
            #[cfg(target_os = "windows")]
            {
                if let Ok(appdata) = std::env::var("APPDATA") {
                    app_data_path = Some(Path::new(&appdata).join("eml_viewer").join("settings.toml"));
                }
            }
            #[cfg(target_os = "macos")]
            {
                if let Ok(home) = std::env::var("HOME") {
                    app_data_path = Some(Path::new(&home).join("Library").join("Application Support").join("eml_viewer").join("settings.toml"));
                }
            }

            if let Some(path) = app_data_path {
                if path.exists() {
                    println!("Loading settings from AppData: {:?}", path);
                    content = fs::read_to_string(&path)?;
                    actual_path = Some(path);
                }
            }
        }

        if let Some(path) = actual_path {
            let mut settings: Settings = toml::from_str(&content)?;
            settings.source_path = Some(path);
            Ok(settings)
        } else {
            // Default settings if file not found
            println!("No settings.toml found, using hardcoded defaults.");
            Ok(Settings {
                zip_path: "data/md_data.mbxc".to_string(),
                filter_labels: Some(vec![]),
                special_labels: Some(vec![
                    "Spam".to_string(),
                    "Papierkorb".to_string(),
                    "Gelöscht".to_string(),
                    "Gesendet".to_string(),
                ]),
                browser: None,
                source_path: None,
            })
        }
    }
}
