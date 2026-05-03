#![windows_subsystem = "windows"]

use std::{env, process::ExitCode};

use sisr::{
    app::{runner::AppRunner, steam},
    config::{CONFIG, Config},
    logging,
};

fn main() -> ExitCode {
    logging::setup();

    #[cfg(windows)]
    {
        sisr::win_console::alloc();
    }

    unsafe {
        // TODO: does this do anything?
        env::set_var("SteamStreamingVideo", "0");
        env::set_var("SteamStreaming", "0");

        env::set_var("SDL_GAMECONTROLLER_ALLOW_STEAM_VIRTUAL_GAMEPAD", "1");
        env::set_var("SDL_JOYSTICK_HIDAPI_STEAMXBOX", "1");
        // this specific SDL_Hint doesn't work when Steam is injected.
        // Envar does...
        env::set_var("SDL_GAMECONTROLLER_IGNORE_DEVICES", "");
        env::set_var("SDL_GAMECONTROLLER_IGNORE_DEVICES_EXCEPT", "");
    }

    let config = Config::parse();
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
                tracing::error!("Failed to parse log file level: {}", e);
            }
        }
    }
    tracing::trace!("merged config: {:?}", config);

    tracing::trace!(
        viiper_min_version = sisr::viiper_metadata::VIIPER_MIN_VERSION,
        viiper_allow_dev = sisr::viiper_metadata::VIIPER_ALLOW_DEV,
        viiper_fetch_prelease = sisr::viiper_metadata::VIIPER_FETCH_PRELEASE,
        "VIIPER metadata"
    );

    tracing::trace!("Environment variables:");
    for (key, value) in env::vars() {
        tracing::trace!("  {}={}", key, value);
    }

    #[cfg(windows)]
    {
        if config.console.unwrap_or(false) {
            sisr::win_console::show();
        }
    }

    // just fill onceLock if we are started via Steam or not.
    steam::util::init();

    let mut app = AppRunner::new();
    let res = app.run();

    #[cfg(windows)]
    {
        sisr::win_console::cleanup();
    }

    res
}
