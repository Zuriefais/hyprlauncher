use freedesktop_entry_parser::parse_entry;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::OnceLock;
use tokio::sync::RwLock;
use walkdir::WalkDir;

pub static APP_CACHE: OnceLock<RwLock<HashMap<String, AppEntry>>> = OnceLock::new();

#[derive(Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub name: String,
    pub exec: String,
    pub icon_name: String,
    pub path: String,
    pub launch_count: u32,
    pub entry_type: EntryType,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum EntryType {
    Application,
    File,
}

pub static HEATMAP_PATH: &str = "~/.local/share/hyprlauncher/heatmap.toml";

pub async fn increment_launch_count(app: &AppEntry) {
    // Initialize the cache if it hasn't been already
    let cache = APP_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
    let mut cache = cache.write().await;
    if let Some(entry) = cache.get_mut(&app.name) {
        entry.launch_count += 1;
        let count = entry.launch_count;
        // Clone the name to avoid lifetime issues
        let app_name = app.name.clone();
        tokio::task::spawn_blocking(move || save_heatmap(&app_name, count));
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Heatmap {
    map: HashMap<String, u32>,
}

fn save_heatmap(name: &str, count: u32) {
    let path = shellexpand::tilde(HEATMAP_PATH).to_string();
    let path = std::path::Path::new(&path);

    // Ensure directory and file exist
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).unwrap_or_default();
    }

    // Create file if it doesn't exist
    if !path.exists() {
        std::fs::File::create(&path).unwrap();
    }

    let mut heatmap = load_heatmap();
    heatmap.map.insert(name.to_string(), count);

    if let Ok(contents) = toml::to_string(&heatmap) {
        fs::write(path, contents).unwrap_or_default();
    }
}

fn load_heatmap() -> Heatmap {
    let path = shellexpand::tilde(HEATMAP_PATH).to_string();
    let path = std::path::Path::new(&path);

    // Create file if it doesn't exist
    if !path.exists() {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).unwrap_or_default();
        }
        std::fs::File::create(&path).unwrap();
        return Heatmap {
            map: HashMap::new(),
        };
    }

    Heatmap {
        map: fs::read_to_string(path)
            .ok()
            .and_then(|contents| toml::from_str(&contents).ok())
            .unwrap_or_default(),
    }
}

pub async fn load_applications() {
    let heatmap = tokio::task::spawn_blocking(load_heatmap)
        .await
        .unwrap_or_default();

    let mut apps = HashMap::new();
    let desktop_paths = [
        "/usr/share/applications",
        "/usr/local/share/applications",
        "~/.local/share/applications",
    ];

    for path in desktop_paths {
        let expanded_path = shellexpand::tilde(path).to_string();
        if let Ok(entries) = std::fs::read_dir(expanded_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".desktop") {
                        if let Ok(desktop_entry) = parse_entry(entry.path()) {
                            if let Some(app_name) =
                                desktop_entry.section("Desktop Entry").attr("Name")
                            {
                                let exec = desktop_entry
                                    .section("Desktop Entry")
                                    .attr("Exec")
                                    .unwrap_or("")
                                    .to_string();
                                let icon = desktop_entry
                                    .section("Desktop Entry")
                                    .attr("Icon")
                                    .unwrap_or("application-x-executable")
                                    .to_string();
                                let launch_count =
                                    heatmap.map.get(app_name).copied().unwrap_or_default();

                                apps.insert(
                                    app_name.to_string(),
                                    AppEntry {
                                        name: app_name.to_string(),
                                        exec,
                                        icon_name: icon,
                                        path: entry.path().to_string_lossy().to_string(),
                                        launch_count,
                                        entry_type: EntryType::Application,
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    let path = std::env::var("PATH").unwrap_or_default();
    let path_entries: Vec<_> = path.split(':').collect();

    let results: Vec<_> = path_entries
        .par_iter()
        .flat_map(|path_entry| {
            WalkDir::new(path_entry)
                .follow_links(true)
                .max_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_type().is_file()
                        && e.metadata()
                            .map(|m| m.permissions().mode() & 0o111 != 0)
                            .unwrap_or(false)
                })
                .filter_map(|entry| {
                    entry.file_name().to_str().map(|name| {
                        let name = name.to_string();
                        let path = entry.path().to_string_lossy().to_string();
                        let launch_count = heatmap.map.get(&name).copied().unwrap_or_default();

                        let icon_name = find_desktop_entry(&name)
                            .map(|e| e.icon_name)
                            .unwrap_or_else(|| "application-x-executable".to_string());

                        (
                            name.clone(),
                            AppEntry {
                                name,
                                exec: path.clone(),
                                icon_name,
                                path,
                                launch_count,
                                entry_type: EntryType::Application,
                            },
                        )
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect();

    for (name, entry) in results {
        apps.insert(name, entry);
    }

    APP_CACHE.set(RwLock::new(apps));
}

struct DesktopEntry {
    icon_name: String,
}

fn find_desktop_entry(name: &str) -> Option<DesktopEntry> {
    let paths = [
        "/usr/share/applications",
        "/usr/local/share/applications",
        "~/.local/share/applications",
    ];

    for path in paths {
        let desktop_file = format!("{}/{}.desktop", path, name);
        if let Ok(entry) = parse_entry(&desktop_file) {
            if let Some(icon) = entry.section("Desktop Entry").attr("Icon") {
                return Some(DesktopEntry {
                    icon_name: icon.to_string(),
                });
            }
        }
    }
    None
}

pub fn create_file_entry(path: String) -> Option<AppEntry> {
    let path = if path.starts_with('~') || path.starts_with('$') {
        shellexpand::full(&path).ok()?.to_string()
    } else {
        path
    };

    let metadata = std::fs::metadata(&path).ok()?;

    if !metadata.is_file() && !metadata.is_dir() {
        return None;
    }

    let name = std::path::Path::new(&path)
        .file_name()?
        .to_str()?
        .to_string();

    let (icon_name, exec) = if metadata.is_dir() {
        ("folder", String::new())
    } else if metadata.permissions().mode() & 0o111 != 0 {
        ("application-x-executable", format!("\"{}\"", path))
    } else {
        let mime_type = match std::process::Command::new("file")
            .arg("--mime-type")
            .arg("-b")
            .arg(&path)
            .output()
        {
            Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
            Err(_) => String::from("application/octet-stream"),
        };

        let icon = match mime_type.split('/').next().unwrap_or("") {
            "text" => "text-x-generic",
            "image" => "image-x-generic",
            "audio" => "audio-x-generic",
            "video" => "video-x-generic",
            "application" => match std::path::Path::new(&path)
                .extension()
                .and_then(|s| s.to_str())
            {
                Some("pdf") => "application-pdf",
                _ => "application-x-generic",
            },
            _ => "text-x-generic",
        };

        (
            icon,
            format!(
                "xdg-mime query default {} | xargs -I {{}} sh -c 'which {{}} >/dev/null && {{}} \"{}\" || xdg-open \"{}\"'",
                mime_type, path, path
            ),
        )
    };

    Some(AppEntry {
        name,
        exec,
        icon_name: icon_name.to_string(),
        path,
        launch_count: 0,
        entry_type: EntryType::File,
    })
}
