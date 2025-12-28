use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};

#[cfg(target_os = "linux")]
use std::rc::Rc;

use tracing::Span;
use tracing::{Level, event, info, span, trace, warn};
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::app::core::get_tokio_handle;
use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::sdl_loop;
use crate::app::steam_utils::binding_enforcer::binding_enforcer;
use crate::app::steam_utils::util::open_controller_config;
use crate::app::window::RunnerEvent;
use crate::app::window::{self, ICON_BYTES};
use crate::config::CONFIG;

use super::core::App;

#[cfg(windows)]
static TRAY_THREAD_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

#[cfg(target_os = "linux")]
static GTK_QUIT_REQUESTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub enum TrayMenuEvent {
    Quit,
    ToggleWindow,
}

struct TrayContext {
    _tray_icon: TrayIcon,
    quit_id: MenuId,
    toggle_window_item: MenuItem,
    toggle_window_id: MenuId,
    open_config_item: MenuItem,
    open_config_id: MenuId,
    force_config_item: CheckMenuItem,
    force_config_id: MenuId,
    kbm_emulation_item: Option<CheckMenuItem>,
    kbm_emulation_id: Option<MenuId>,
    window_visible: Arc<Mutex<bool>>,
    ui_visible: Arc<Mutex<bool>>,
    fullscreen: bool,
}

impl TrayContext {
    fn new(window_visible: Arc<Mutex<bool>>, ui_visible: Arc<Mutex<bool>>) -> Self {
        let icon = load_icon();
        let menu = Menu::new();

        let fullscreen = CONFIG
            .read()
            .ok()
            .and_then(|c| c.as_ref().map(|cfg| cfg.window.fullscreen.unwrap_or(true)))
            .unwrap_or(true);
        let initial_visible = *window_visible.lock().unwrap();
        let initial_ui_visible = if fullscreen {
            *ui_visible.lock().unwrap()
        } else {
            initial_visible
        };
        let initial_text = if fullscreen {
            if initial_ui_visible {
                "Hide UI"
            } else {
                "Show UI"
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if initial_visible {
                "Hide Window"
            } else {
                "Show Window"
            }
        };
        let toggle_window_item = MenuItem::new(initial_text, true, None);
        let toggle_window_id = toggle_window_item.id().clone();
        menu.append(&toggle_window_item)
            .expect("Failed to add toggle window item");

        let has_app_id = binding_enforcer()
            .lock()
            .ok()
            .and_then(|e| e.app_id())
            .is_some();
        let open_config_item = MenuItem::new("Steam Controllerconfig", has_app_id, None);
        let open_config_id = open_config_item.id().clone();
        menu.append(&open_config_item)
            .expect("Failed to add open configurator item");

        let force_config_item = CheckMenuItem::new("Force Controllerconfig", true, false, None);
        let force_config_id = force_config_item.id().clone();
        menu.append(&force_config_item)
            .expect("Failed to add force config item");

        let initial_kbm_enabled = window::is_kbm_emulation_enabled();

        let (kbm_emulation_item, kbm_emulation_id) = {
            let viiper_address = CONFIG
                .read()
                .ok()
                .and_then(|c| c.as_ref().and_then(|cfg| cfg.viiper_address.clone()));
            let viiper_is_loopback = viiper_address
                .as_ref()
                .and_then(|addr_str| addr_str.to_socket_addrs().ok())
                .and_then(|mut addrs| addrs.next())
                .map(|addr| addr.ip().is_loopback())
                .unwrap_or(true);

            if !viiper_is_loopback {
                let item =
                    CheckMenuItem::new("Keyboard/mouse emulation", true, initial_kbm_enabled, None);
                let id = item.id().clone();
                menu.append(&item)
                    .expect("Failed to add kb/m emulation item");
                (Some(item), Some(id))
            } else {
                (None, None)
            }
        };

        let quit_item = MenuItem::new("Quit", true, None);
        let quit_id = quit_item.id().clone();
        menu.append(&quit_item).expect("Failed to add quit item");

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("SISR")
            .with_icon(icon)
            .build()
            .expect("Failed to create tray icon");

        Self {
            _tray_icon: tray_icon,
            quit_id,
            toggle_window_item,
            toggle_window_id,
            open_config_item,
            open_config_id,
            force_config_item,
            force_config_id,
            kbm_emulation_item,
            kbm_emulation_id,
            window_visible,
            ui_visible,
            fullscreen,
        }
    }

    fn handle_events(&self) -> bool {
        if let Ok(guard) = binding_enforcer().lock() {
            self.open_config_item.set_enabled(guard.app_id().is_some());
            self.force_config_item.set_checked(guard.is_active());
        } else {
            warn!("Failed to acquire binding enforcer lock to update open configurator menu item");
        }

        if let Some(item) = &self.kbm_emulation_item {
            item.set_checked(window::is_kbm_emulation_enabled());
        }

        let (menu_text, _is_fullscreen) = if self.fullscreen {
            if let Ok(guard) = self.ui_visible.lock() {
                let ui_vis = *guard;
                (if ui_vis { "Hide UI" } else { "Show UI" }, true)
            } else {
                ("Show UI", true)
            }
        } else if let Ok(guard) = self.window_visible.lock() {
            let vis = *guard;
            (if vis { "Hide Window" } else { "Show Window" }, false)
        } else {
            ("Show Window", false)
        };
        self.toggle_window_item.set_text(menu_text);

        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.quit_id {
                info!("Quit requested from tray menu");
                App::shutdown();
                return true;
            }
            if event.id == self.toggle_window_id {
                if self.fullscreen {
                    _ = window::get_event_sender().send_event(RunnerEvent::ToggleUi());
                } else {
                    let visible = if let Ok(mut guard) = self.window_visible.lock() {
                        *guard = !*guard;
                        *guard
                    } else {
                        false
                    };
                    let event = if visible {
                        RunnerEvent::ShowWindow()
                    } else {
                        RunnerEvent::HideWindow()
                    };
                    _ = window::get_event_sender().send_event(event);
                }

                return false;
            }
            if event.id == self.open_config_id {
                if let Ok(guard) = binding_enforcer().lock()
                    && let Some(app_id) = guard.app_id()
                {
                    get_tokio_handle().spawn(open_controller_config(app_id));
                }
                return false;
            }
            if event.id == self.force_config_id {
                if let Ok(mut guard) = binding_enforcer().lock() {
                    if guard.is_active() {
                        guard.deactivate();
                    } else {
                        guard.activate();
                    }
                }
                return false;
            }

            if let Some(kbm_id) = &self.kbm_emulation_id
                && event.id == *kbm_id
                && let Some(item) = &self.kbm_emulation_item
            {
                let enabled = !window::is_kbm_emulation_enabled();
                window::set_kbm_emulation_enabled(enabled);
                item.set_checked(enabled);

                _ = sdl_loop::get_event_sender().push_custom_event(HandlerEvent::SetKbmEmulation {
                    enabled,
                    initialize: false,
                });

                return false;
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
            use tracing::trace;

            unsafe {
                PostThreadMessageW(thread_id, WM_QUIT, 0, 0);
            }
            trace!("Posted WM_QUIT to tray thread");
        }
    }

    #[cfg(target_os = "linux")]
    {
        GTK_QUIT_REQUESTED.store(true, std::sync::atomic::Ordering::SeqCst);
        glib::idle_add(|| {
            gtk::main_quit();
            glib::ControlFlow::Break
        });
        trace!("Set GTK quit flag and scheduled main_quit");
    }
}

pub fn run(window_visible: Arc<Mutex<bool>>, ui_visible: Arc<Mutex<bool>>) {
    let span = span!(Level::INFO, "tray");
    run_platform(span, window_visible, ui_visible);
}

#[cfg(windows)]
fn run_platform(span: Span, window_visible: Arc<Mutex<bool>>, ui_visible: Arc<Mutex<bool>>) {
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, MSG, TranslateMessage, WM_QUIT,
    };

    let thread_id = unsafe { GetCurrentThreadId() };
    TRAY_THREAD_ID.store(thread_id, std::sync::atomic::Ordering::SeqCst);

    let ctx = TrayContext::new(window_visible, ui_visible.clone());

    loop {
        if ctx.handle_events() {
            event!(parent: &span, Level::DEBUG, "Tray context requested quit, exiting tray loop");
            break;
        }

        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            let ret = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
            if ret == 0 || ret == -1 || msg.message == WM_QUIT {
                event!(parent: &span, Level::DEBUG, "Received WM_QUIT or error in GetMessageW, exiting tray loop");
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[cfg(target_os = "linux")]
fn run_platform(span: Span, window_visible: Arc<Mutex<bool>>, ui_visible: Arc<Mutex<bool>>) {
    if gtk::init().is_err() {
        event!(parent: &span, Level::ERROR, "Failed to initialize GTK for tray icon");
        return;
    }

    let ctx = Rc::new(TrayContext::new(window_visible, ui_visible.clone()));
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        if GTK_QUIT_REQUESTED.load(std::sync::atomic::Ordering::SeqCst) {
            gtk::main_quit();
            return glib::ControlFlow::Break;
        }
        if ctx.handle_events() {
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
