use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub width: i32,
    pub height: i32,
    pub font_size: i32,
    pub theme: Theme,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Theme {
    pub background_color: String,
    pub text_color: String,
    pub selection_color: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 600,
            height: 600,
            font_size: 14,
            theme: Theme {
                background_color: String::from("#0f0f0f"),
                text_color: String::from("#eceff4"),
                selection_color: String::from("#1f1f1f"),
            },
        }
    }
}

impl Config {
    fn ensure_config_dir() -> PathBuf {
        let config_path = config_dir()
            .map(|mut p| {
                p.push("hyprlauncher");
                p
            })
            .unwrap_or_else(|| PathBuf::from("~/.config/hyprlauncher"));

        if !config_path.exists() {
            fs::create_dir_all(&config_path).unwrap_or_default();
        }

        let css_path = config_path.join("style.css");
        if !css_path.exists() {
            let default_config = Config::default();
            fs::write(&css_path, get_default_css(&default_config)).unwrap_or_default();
        }

        config_path
    }

    pub fn load() -> Self {
        let config_path = Self::ensure_config_dir();
        let config_file = config_path.join("config.toml");
        let default_config = Config::default();

        if !config_file.exists() {
            if let Ok(contents) = toml::to_string_pretty(&default_config) {
                fs::write(&config_file, contents).unwrap_or_default();
            }
            return default_config;
        }

        let config_str = fs::read_to_string(&config_file).unwrap_or_default();
        let config: Config = toml::from_str(&config_str).unwrap_or_else(|_| default_config);

        config
    }

    pub fn load_css() -> String {
        let config_path = Self::ensure_config_dir();
        let css_path = config_path.join("style.css");
        let config = Self::load();
        fs::read_to_string(css_path).unwrap_or_else(|_| get_default_css(&config))
    }
}

fn get_default_css(config: &Config) -> String {
    format!(
        "window {{
            background-color: #0f0f0f;
        }}

        list {{
            background: #0f0f0f;
        }}

        list row {{
            padding: 4px;
            margin: 2px 6px;
            border-radius: 8px;
            background: #0f0f0f;
            transition: all 200ms ease;
        }}

        list row:selected {{
            background-color: {};
        }}

        list row:hover:not(:selected) {{
            background-color: #181818;
        }}

        entry {{
            margin: 12px;
            margin-bottom: 8px;
            padding: 12px;
            border-radius: 8px;
            background-color: #1f1f1f;
            color: #e0e0e0;
            caret-color: #808080;
            font-size: 16px;
            box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
        }}

        entry:focus {{
            background-color: #282828;
        }}

        .app-name {{
            color: #ffffff;
            font-size: 14px;
            font-weight: bold;
            margin-right: 8px;
        }}

        .app-description {{
            color: #a0a0a0;
            font-size: 12px;
            margin-right: 8px;
        }}

        .app-path {{
            color: #808080;
            font-size: 12px;
            font-family: monospace;
            opacity: 0.8;
        }}",
        config.theme.selection_color
    )
}
