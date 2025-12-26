#![windows_subsystem = "windows"]

use std::{env, process::ExitCode};

use sisr::{
    app::steam_utils::{self},
    config::CONFIG,
    logging,
};
use tracing::{error, info, trace};

fn main() -> ExitCode {
    logging::setup();

    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "--create-cef-file" {
        info!("Creating Steam CEF debug enable file at: {}", args[2]);
        match std::fs::File::create(&args[2]) {
            Ok(_) => {
                info!("CEF debug file created successfully");
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "SISR") {
                    let error_file = proj_dirs
                        .data_dir()
                        .parent()
                        .unwrap()
                        .join("cef_creation_error.txt");
                    if let Some(parent) = error_file.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::write(
                        &error_file,
                        format!(
                            "Failed to create CEF file: {}\nAttempted path: {}",
                            e, &args[2]
                        ),
                    );
                }
                return ExitCode::FAILURE;
            }
        }
    }

    #[cfg(windows)]
    {
        sisr::win_console::alloc();
    }

    unsafe {
        // TODO: does this do anything?
        env::set_var("SteamStreamingVideo", "0");
        env::set_var("SteamStreaming", "0");

        env::set_var("SDL_GAMECONTROLLER_ALLOW_STEAM_VIRTUAL_GAMEPAD", "1");
        env::set_var("SDL_JOYSTICK_HIDAPI_STEAMXBOX", "0");
        // this specific SDL_Hint doesn't work when Steam is injected.
        // Envar does...
        env::set_var("SDL_GAMECONTROLLER_IGNORE_DEVICES", "");
    }

    info!("Starting SISR...");

    steam_utils::binding_enforcer::install_cleanup_handlers();

    let config = sisr::config::Config::parse();
    *CONFIG.write().unwrap() = Some(config.clone());

    logging::set_level(config.log.level.as_ref().unwrap().parse().unwrap());

    if let Some(log_file) = &config.log.log_file
        && let Some(path) = &log_file.path
    {
        match log_file
            .file_level
            .as_ref()
            .unwrap_or(&config.log.level.as_ref().unwrap().parse().unwrap())
            .parse()
        {
            Ok(level) => logging::add_file(path, level),
            Err(e) => {
                error!("Failed to parse log file level: {}", e);
            }
        }
    }
    trace!("merged config: {:?}", config);

    trace!(
        viiper_min_version = sisr::viiper_metadata::VIIPER_MIN_VERSION,
        viiper_allow_dev = sisr::viiper_metadata::VIIPER_ALLOW_DEV,
        viiper_fetch_prelease = sisr::viiper_metadata::VIIPER_FETCH_PRELEASE,
        "VIIPER metadata"
    );

    // ADD ENV TRACE LOGGING HERE!
    trace!("Environment variables:");
    for (key, value) in env::vars() {
        trace!("  {}={}", key, value);
    }

    #[cfg(windows)]
    {
        if config.console.unwrap_or(false) {
            sisr::win_console::show();
        }
    }

    // just fill onceLock if we are started via Steam or not.
    steam_utils::util::init();

    let mut app = sisr::app::App::new();
    let result = app.run();

    #[cfg(windows)]
    {
        sisr::win_console::cleanup();
    }

    result
}
