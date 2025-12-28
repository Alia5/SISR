use std::sync::{Arc, Mutex, OnceLock};
use tracing::{debug, error, info, warn};

use crate::app::{
    signals,
    steam_utils::util::{launched_in_steam_game_mode, open_steam_url},
};

#[derive(Debug)]
pub struct BindingEnforcer {
    game_id: Option<u64>,
    app_id: Option<u32>,
    active: bool,
}

impl BindingEnforcer {
    pub fn new() -> Self {
        let game_id = std::env::var("SteamGameId")
            .ok()
            .and_then(|s| s.parse::<u64>().ok());

        let app_id = game_id.map(|gid| (gid >> 32) as u32);

        if let Some(gid) = game_id {
            info!("Detected SteamGameId: {}", gid);
        }
        if let Some(aid) = app_id {
            info!("Calculated AppId: {}", aid);
        }

        Self {
            game_id,
            app_id,
            active: false,
        }
    }

    pub fn game_id(&self) -> Option<u64> {
        self.game_id
    }

    pub fn app_id(&self) -> Option<u32> {
        self.app_id
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn activate(&mut self) {
        if self.active {
            debug!("Steam binding enforcement already active");
            return;
        }

        let Some(app_id) = self.app_id else {
            warn!("Cannot activate Steam binding enforcement: no AppId detected");
            return;
        };

        if launched_in_steam_game_mode() {
            info!("Skipping overlay enforcement: Launched in Steam Game Mode");
            return;
        }

        let url = format!("steam://forceinputappid/{}", app_id);
        match open_steam_url(&url) {
            Ok(_) => {
                info!("Activated Steam binding enforcement for AppId: {}", app_id);
                self.active = true;
            }
            Err(e) => {
                error!("Failed to activate Steam binding enforcement: {}", e);
            }
        }
    }

    pub fn activate_with_appid(&mut self, app_id: u32) {
        if self.active {
            debug!("Steam binding enforcement already active");
            return;
        }

        let url = format!("steam://forceinputappid/{}", app_id);
        match open_steam_url(&url) {
            Ok(_) => {
                info!("Activated Steam binding enforcement for AppId: {}", app_id);
                self.active = true;
            }
            Err(e) => {
                error!("Failed to activate Steam binding enforcement: {}", e);
            }
        }
    }

    pub fn deactivate(&mut self) {
        if !self.active {
            debug!("Steam binding enforcement already inactive");
            return;
        }

        match open_steam_url("steam://forceinputappid/0") {
            Ok(_) => {
                info!("Deactivated Steam binding enforcement");
                self.active = false;
            }
            Err(e) => {
                error!("Failed to deactivate Steam binding enforcement: {}", e);
            }
        }
        // steam sometimes takes ages to deactive, sending twice helps ¯\_(ツ)_/¯
        match open_steam_url("steam://forceinputappid/0") {
            Ok(_) => {
                self.active = false;
            }
            Err(e) => {}
        }
    }
}

impl Default for BindingEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for BindingEnforcer {
    fn drop(&mut self) {
        if self.active {
            self.deactivate();
        }
    }
}

pub fn install_cleanup_handlers() {
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = open_steam_url("steam://forceinputappid/0").inspect_err(|e| {
            error!(
                "Failed to cleanup Steam binding enforcement on panic: {}",
                e
            );
        });
        original_hook(panic_info);
    }));

    if let Err(e) = signals::register_ctrlc_handler(move || {
        let _ = open_steam_url("steam://forceinputappid/0").inspect_err(|e| {
            error!(
                "Failed to cleanup Steam binding enforcement on Ctrl+C: {}",
                e
            );
        });
    }) {
        warn!("Failed to install Steam cleanup Ctrl+C handler: {}", e);
    }
}

static BINDING_ENFORCER: OnceLock<Arc<Mutex<BindingEnforcer>>> = OnceLock::new();

pub fn binding_enforcer() -> &'static Arc<Mutex<BindingEnforcer>> {
    BINDING_ENFORCER.get_or_init(|| Arc::new(Mutex::new(BindingEnforcer::new())))
}
