#![windows_subsystem = "windows"]
// TODO: don't force this via src file. (do rustflags in CI)

use std::process::ExitCode;

use sisr::{app::steam_utils, logging};
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

    info!("Starting SISR...");

    steam_utils::binding_enforcer::install_cleanup_handlers();

    let config = sisr::config::Config::parse();
    sisr::config::CONFIG.set(config.clone()).unwrap();

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
