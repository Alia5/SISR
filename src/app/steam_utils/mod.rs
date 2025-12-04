mod binding_enforcer;

pub use binding_enforcer::BindingEnforcer;

use std::process::Command;
use tracing::debug;

pub fn open_steam_url(url: &str) -> Result<(), std::io::Error> {
    debug!("Opening Steam URL: {}", url);

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd").args(["/c", "start", "", url]).spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(url).spawn()?;
    }

    Ok(())
}

pub fn install_cleanup_handlers() {
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = open_steam_url("steam://forceinputappid/0");
        original_hook(panic_info);
    }));

    let _ = ctrlc::set_handler(move || {
        let _ = open_steam_url("steam://forceinputappid/0");
        std::process::exit(0);
    });
}
