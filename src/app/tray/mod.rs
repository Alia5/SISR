pub mod event;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg(target_os = "linux")]
use std::rc::Rc;

use tokio::sync::mpsc;
use tracing::{Level, Span};
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::app::assets::ICON_BYTES;
use crate::app::input::context::Context;
use crate::app::steam::binding_enforcer::binding_enforcer;
use crate::app::tray::event::TrayEvent;
use crate::app::{steam, updater};
use crate::app::window::event::{WindowRunnerEvent, get_event_sender};
use crate::config::get_config;

use super::runner::{AppRunner, get_tokio_handle};

#[cfg(windows)]
static TRAY_THREAD_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

#[cfg(target_os = "linux")]
static GTK_QUIT_REQUESTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub enum TrayMenuEvent {
    Quit,
    ToggleUI,
    ToggleSteamOverlayEnabled,
    OpenControllerConfig,
    ToggleForceControllerConfig,
    ShowUpdateDialog,
}

pub struct TrayContext {
    tray_icon: TrayIcon,
    menu_ids: HashMap<MenuId, TrayMenuEvent>,
    receiver: mpsc::UnboundedReceiver<TrayEvent>,
    version_item: MenuItem,
    toggle_ui_item: MenuItem,
    enable_steam_overlay_item: CheckMenuItem,
    open_config_item: Option<MenuItem>,
    force_config_item: CheckMenuItem,
    quit_item: MenuItem,
    update_item: Option<MenuItem>,
    ctx: Arc<Mutex<Context>>,
}

impl TrayContext {
    pub fn new(ctx: Arc<Mutex<Context>>) -> Self {
        let icon = load_icon();
        let menu = Menu::new();

        let mut menu_ids = HashMap::new();

        let version_item =
            MenuItem::new(format!("SISR v{}", env!("CARGO_PKG_VERSION")), false, None);
        menu.append(&version_item)
            .expect("Failed to add version entry");

        let toggle_ui_item = MenuItem::new("Hide UI", true, None);
        menu.append(&toggle_ui_item)
            .expect("Failed to add toggle window item");
        menu_ids.insert(toggle_ui_item.id().clone(), TrayMenuEvent::ToggleUI);

        let enable_steam_overlay_item = CheckMenuItem::new(
            "Enable Steam Overlay",
            true,
            get_config().window.create.unwrap_or(false) && get_config().window.fullscreen.unwrap_or(true),
            None,
        );
        menu.append(&enable_steam_overlay_item)
            .expect("Failed to add enable steam overlay item");
        menu_ids.insert(enable_steam_overlay_item.id().clone(), TrayMenuEvent::ToggleSteamOverlayEnabled);

        let enforcer = binding_enforcer().lock().ok();
        let app_id = enforcer.as_ref().and_then(|e| e.app_id());
        let enforcer_active = enforcer.as_ref().map(|e| e.is_active()).unwrap_or(false);
        drop(enforcer);

        let open_config_item = app_id.map(|_| {
            let item = MenuItem::new("Show Steam Controllerconfig", true, None);
            menu.append(&item)
                .expect("Failed to add Steam Controllerconfig item");
            menu_ids.insert(item.id().clone(), TrayMenuEvent::OpenControllerConfig);
            item
        });

        let force_config_item = CheckMenuItem::new(
            "Allow Desktop Config",
            app_id.is_some(),
            !enforcer_active,
            None,
        );
        menu.append(&force_config_item)
            .expect("Failed to add Force Controllerconfig item");
        menu_ids.insert(force_config_item.id().clone(), TrayMenuEvent::ToggleForceControllerConfig);

        let quit_item = MenuItem::new("Quit", true, None);
        menu.append(&quit_item).expect("Failed to add quit item");
        menu_ids.insert(quit_item.id().clone(), TrayMenuEvent::Quit);

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("SISR")
            .with_icon(icon)
            .build()
            .expect("Failed to create tray icon");

        let (tx, rx) = mpsc::unbounded_channel();
        event::init(tx);

        Self {
            tray_icon,
            menu_ids,
            receiver: rx,
            version_item,
            toggle_ui_item,
            enable_steam_overlay_item,
            open_config_item,
            force_config_item,
            quit_item,
            update_item: None,
            ctx,
        }
    }

    fn rebuild_menu(&mut self) {
        let menu = Menu::new();
        if let Some(upd) = &self.update_item {
            menu.append(upd).ok();
        }
        menu.append(&self.version_item).ok();
        menu.append(&self.toggle_ui_item).ok();
        menu.append(&self.enable_steam_overlay_item).ok();
        if let Some(item) = &self.open_config_item {
            menu.append(item).ok();
        }
        menu.append(&self.force_config_item).ok();
        menu.append(&self.quit_item).ok();
        self.tray_icon.set_menu(Some(Box::new(menu)));
    }

    pub fn handle_events(&mut self) -> bool {
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            match self.menu_ids[&event.id] {
                TrayMenuEvent::Quit => {
                    tracing::info!("Quit requested from tray menu");
                    AppRunner::shutdown();
                    return true;
                }
                TrayMenuEvent::ToggleUI => {
                    tracing::debug!("Toggle window requested from tray menu");
                    let currently_visible = self.ctx.lock().map(|c| c.ui_visible).unwrap_or(false);
                    let show = !currently_visible;
                    if let Err(e) = get_event_sender().send_event(WindowRunnerEvent::ToggleUi(Some(show))) {
                        tracing::error!("Failed to send ToggleUi event: {:?}", e);
                    }
                }
                TrayMenuEvent::ToggleSteamOverlayEnabled => {
                    tracing::debug!("Toggle Steam Overlay requested from tray menu");
                    let cfg = get_config();
                    let currently_enabled = cfg.window.create.unwrap_or(false) && cfg.window.fullscreen.unwrap_or(true);
                    let fullscreen = !currently_enabled;
                    self.enable_steam_overlay_item.set_checked(fullscreen);
                    if let Err(e) = get_event_sender().send_event(WindowRunnerEvent::SetFullscreen(fullscreen)) {
                        tracing::error!("Failed to send SetFullscreen event: {:?}", e);
                    }
                    if fullscreen {
                        if let Err(e) = get_event_sender().send_event(WindowRunnerEvent::ShowWindow()) {
                            tracing::error!("Failed to send ShowWindow event: {:?}", e);
                        }
                    } else {
                        if let Err(e) = get_event_sender().send_event(WindowRunnerEvent::HideWindow()) {
                            tracing::error!("Failed to send HideWindow event: {:?}", e);
                        }
                    }
                }
                TrayMenuEvent::OpenControllerConfig => {
                    tracing::debug!("Open Steam Controllerconfig requested from tray menu");
                    if let Some(app_id) = binding_enforcer().lock().ok().and_then(|e| e.app_id()) {
                        get_tokio_handle().spawn(async move {
                            steam::util::open_controller_config(app_id).await;
                        });
                    }
                }
                TrayMenuEvent::ToggleForceControllerConfig => {
                    tracing::debug!("Toggle Force Controllerconfig requested from tray menu");
                    if let Ok(mut enforcer) = binding_enforcer().lock() {
                        if enforcer.is_active() {
                            enforcer.deactivate();
                        } else {
                            enforcer.activate();
                        }
                        // checkbox is inverted: checked = desktop config allowed = NOT enforcing
                        self.force_config_item.set_checked(!enforcer.is_active());
                    }
                }
                TrayMenuEvent::ShowUpdateDialog => {
                    tracing::debug!("Show update dialog requested from tray menu");
                    updater::clear_dismissed();
                    updater::clear_remind_later();
                    if let Err(e) = get_event_sender().send_event(WindowRunnerEvent::ToggleUi(Some(true))) {
                        tracing::error!("Failed to send ToggleUi event: {:?}", e);
                    }
                    if let Err(e) = get_event_sender().send_event(WindowRunnerEvent::InvalidateSvelteState()) {
                        tracing::error!("Failed to send InvalidateSvelteState event: {:?}", e);
                    }
                }
            }
        }
        while let Ok(evt) = self.receiver.try_recv() {
            tracing::trace!("Received event: {:?}", evt);
            match evt {
                TrayEvent::SetWindowState(visible) => {
                    if visible {
                        self.toggle_ui_item.set_text("Hide UI");
                    } else {
                        self.toggle_ui_item.set_text("Show UI");
                    };
                }
                TrayEvent::ForceConfigChanged(active) => {
                    // checked = desktop config allowed = NOT forcing
                    self.force_config_item.set_checked(!active);
                }
                TrayEvent::UpdateAvailable(version) => {
                    tracing::info!("Update available notification received: {}", version);
                    let update_item = MenuItem::new(
                        format!("\u{2B06} Update available: {}", version),
                        true,
                        None,
                    );
                    self.menu_ids
                        .insert(update_item.id().clone(), TrayMenuEvent::ShowUpdateDialog);
                    self.update_item = Some(update_item);
                    self.rebuild_menu();
                }
            }
        }
        false
    }
}

pub fn shutdown() {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_QUIT};
        let thread_id = TRAY_THREAD_ID.load(std::sync::atomic::Ordering::SeqCst);
        if thread_id != 0 {
            unsafe {
                PostThreadMessageW(thread_id, WM_QUIT, 0, 0);
            }
            tracing::trace!("Posted WM_QUIT to tray thread");
        }
    }

    #[cfg(target_os = "linux")]
    {
        GTK_QUIT_REQUESTED.store(true, std::sync::atomic::Ordering::SeqCst);
        tracing::trace!("Set GTK quit flag");
    }
}

pub fn run(ctx: Arc<Mutex<Context>>) {
    let span = tracing::span!(Level::INFO, "tray");
    run_platform(span, ctx);
}

#[cfg(windows)]
fn run_platform(span: Span, ctx: Arc<Mutex<Context>>) {
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, MSG, TranslateMessage, WM_QUIT,
    };

    let thread_id = unsafe { GetCurrentThreadId() };
    TRAY_THREAD_ID.store(thread_id, std::sync::atomic::Ordering::SeqCst);

    let mut tray_ctx = TrayContext::new(ctx);

    loop {
        if tray_ctx.handle_events() {
            tracing::event!(parent: &span, Level::DEBUG, "Tray context requested quit, exiting tray loop");
            break;
        }

        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            let ret = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
            if ret == 0 || ret == -1 || msg.message == WM_QUIT {
                tracing::event!(parent: &span, Level::DEBUG, "Received WM_QUIT or error in GetMessageW, exiting tray loop");
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[cfg(target_os = "linux")]
fn run_platform(span: Span, ctx: Arc<Mutex<Context>>) {
    use std::cell::RefCell;

    if gtk::init().is_err() {
        tracing::event!(parent: &span, Level::ERROR, "Failed to initialize GTK for tray icon");
        return;
    }

    let tray_ctx = Rc::new(RefCell::new(TrayContext::new(ctx)));
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        if GTK_QUIT_REQUESTED.load(std::sync::atomic::Ordering::SeqCst) {
            gtk::main_quit();
            return glib::ControlFlow::Break;
        }
        if tray_ctx.borrow_mut().handle_events() {
            gtk::main_quit();
            return glib::ControlFlow::Break;
        }
        glib::ControlFlow::Continue
    });

    gtk::main();
}

fn load_icon() -> Icon {
    let image = image::load_from_memory(ICON_BYTES)
        .expect("Failed to load icon")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    Icon::from_rgba(rgba, width, height).expect("Failed to create icon")
}
