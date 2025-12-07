use std::{env, path::PathBuf, sync};

use clap::Parser;
use figment::{
    Figment,
    providers::{Format, Json, Serialized, Toml, Yaml},
};
use serde::{Deserialize, Serialize};
use tracing::debug;

pub static CONFIG: sync::OnceLock<Config> = sync::OnceLock::new();

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
#[command(version, about, long_about = None)]
pub struct Config {
    #[serde(skip)]
    #[arg(
        short = 'c',
        long = "config",
        value_name = "FILE",
        help = "Path to config file"
    )]
    pub config_file_path: Option<PathBuf>,

    #[cfg(windows)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(
        long = "console",
        visible_alias = "cli",
        value_name = "BOOL",
        num_args = 0..=1,
        default_missing_value = "true",
        help = "Show console window (Windows only) (true/false) [default: false]"
    )]
    pub console: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(
        short = 't',
        long = "tray",
        value_name = "BOOL",
        num_args = 0..=1,
        default_missing_value = "true",
        env = "SISR_TRAY",
        help = "Enable system tray icon (true/false) [default: true]"
    )]
    pub tray: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(
        long = "viiper-address",
        env = "SISR_VIIPER_ADDRESS",
        help = "VIIPER API-server address [default: localhost:3242]"
    )]
    pub viiper_address: Option<String>,

    #[command(flatten)]
    pub window: WindowOpts,

    #[command(flatten)]
    pub log: LogOpts,

    #[command(flatten)]
    pub steam: SteamOpts,

    #[serde(skip)]
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,
}

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
pub struct WindowOpts {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(
        short = 'w',
        long = "window-create",
        value_name = "BOOL",
        num_args = 0..=1,
        default_missing_value = "true",
        env = "SISR_WINDOW_CREATE",
        help = "Create a transparent window (true/false) [default: false]"
    )]
    pub create: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(
        short = 'f',
        long = "window-fullscreen",
        value_name = "BOOL",
        num_args = 0..=1,
        default_missing_value = "true",
        env = "SISR_WINDOW_FULLSCREEN",
        help = "Create a fullscreen window [default: true]"
    )]
    pub fullscreen: Option<bool>,
}

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
pub struct LogOpts {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(
        short = 'l',
        long = "log-level",
        value_name = "LEVEL",
        env = "SISR_LOG_LEVEL",
        help = "Set the logging level (error, warn, info, debug, trace) [default: info]"
    )]
    pub level: Option<String>,

    #[command(flatten)]
    pub log_file: Option<LogFile>,
}

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
pub struct LogFile {
    #[serde()]
    #[arg(
        long = "log-file",
        value_name = "FILE",
        env = "SISR_LOG_FILE",
        help = "Path to log file"
    )]
    pub path: Option<PathBuf>,
    #[serde(default, alias = "level", skip_serializing_if = "Option::is_none")]
    pub file_level: Option<String>,
}

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
pub struct SteamOpts {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(
        long = "disable-steam-cef-debug",
        value_name = "BOOL",
        num_args = 0..=1,
        default_missing_value = "false",
        env = "SISR_STEAM_CEF_DEBUG_ENABLE",
        help = "Enable Steam CEF remote debugging (true/false) [default: false]"
    )]
    pub cef_debug_disable: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(
        long = "steam-launch-timeout-secs",
        value_name = "SECONDS",
        env = "SISR_STEAM_LAUNCH_TIMEOUT_SECS",
        help = "Time to wait for Steam to launch in seconds [default: 1]"
    )]
    pub steam_launch_timeout_secs: Option<u64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(
        long = "steam-path",
        value_name = "PATH",
        env = "SISR_STEAM_PATH",
        help = "Path to Steam (if not autodetect)"
    )]
    pub steam_path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_file_path: None,
            #[cfg(windows)]
            console: Some(false),
            tray: Some(true),
            viiper_address: Some("localhost:3242".to_string()),
            window: WindowOpts {
                create: Some(false),
                fullscreen: Some(true),
            },
            log: LogOpts {
                level: if cfg!(debug_assertions) {
                    tracing::Level::DEBUG
                } else {
                    tracing::Level::INFO
                }
                .to_string()
                .into(),
                log_file: None,
            },
            steam: SteamOpts {
                cef_debug_disable: Some(false),
                steam_launch_timeout_secs: Some(1),
                steam_path: None,
            },
            debug: 0,
        }
    }
}

impl Config {
    fn config_candidate_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let config_names = ["SISR", "config"];
        let extensions = ["toml", "yaml", "yml", "json"];

        if let Some(config_dir) = directories::ProjectDirs::from("", "", "SISR") {
            let config_path = config_dir.config_dir();
            for name in &config_names {
                for ext in &extensions {
                    paths.push(config_path.join(format!("{}.{}", name, ext)));
                }
            }
        }

        if let Ok(exe_path) = env::current_exe()
            && let Some(exe_dir) = exe_path.parent()
        {
            for name in &config_names {
                for ext in &extensions {
                    paths.push(exe_dir.join(format!("{}.{}", name, ext)));
                }
            }
        } else {
            debug!("Failed to get current executable path for config search");
        }

        paths
    }

    pub fn parse() -> Self {
        let cli_args = <Self as Parser>::parse();

        let candidates = Self::config_candidate_paths();
        debug!("Config candidate paths: {:?}", candidates);

        let mut cfg = Figment::from(Serialized::defaults(Config::default()));
        for path in candidates {
            cfg = cfg.merge({
                match path.extension().and_then(|s| s.to_str()) {
                    Some("toml") => Figment::from(Toml::file(&path)),
                    Some("yaml" | "yml") => Figment::from(Yaml::file(&path)),
                    Some("json") => Figment::from(Json::file(&path)),
                    _ => Figment::from(Toml::file(&path)),
                }
            });
        }

        match cfg
            .merge({
                match &cli_args.config_file_path {
                    None => Figment::new(),
                    Some(path) => match path.extension().and_then(|s| s.to_str()) {
                        Some("toml") => Figment::from(Toml::file(path)),
                        Some("yaml" | "yml") => Figment::from(Yaml::file(path)),
                        Some("json") => Figment::from(Json::file(path)),
                        _ => Figment::from(Toml::file(path)),
                    },
                }
            })
            .merge(Serialized::defaults(&cli_args))
            .extract()
        {
            Ok(cfg) => cfg,
            Err(e) => {
                panic!("Failed to parse configuration: {}", e);
            }
        }
    }
}
