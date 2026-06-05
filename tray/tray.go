package tray

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"runtime"
	"strconv"

	"fyne.io/systray"
	"github.com/Alia5/SISR/assets"
	"github.com/Alia5/SISR/cefpayloads"
	"github.com/Alia5/SISR/cmd"
	"github.com/Alia5/SISR/meta"
	"github.com/Alia5/SISR/sdl"
	"github.com/Alia5/SISR/sdl/extras"
	"github.com/Alia5/SISR/webview"
)

type TrayNotifyCh chan any

type UpdateAvailableNotification struct {
	Version string
}

func Run(ctx context.Context, c *cmd.SISRContext, trayNotifyCh TrayNotifyCh) {
	start, end := systray.RunWithExternalLoop(
		func() {
			onReady(ctx, c, trayNotifyCh)
		},
		func() {},
	)
	start()
	go func() {
		<-ctx.Done()
		end()
	}()
}

func onReady(ctx context.Context, c *cmd.SISRContext, trayNotifyCh TrayNotifyCh) {
	if runtime.GOOS == "windows" {
		systray.SetIcon(assets.IconICO)
	} else {
		systray.SetIcon(assets.IconPNG)
	}
	systray.SetTooltip("SISR - Steam Input System Redirector")

	infoStr := fmt.Sprintf("SISR - %s", meta.Version)
	versionItem := systray.AddMenuItem(infoStr, infoStr)
	versionItem.Disable()

	updateAvailableItem := systray.AddMenuItem("", "")
	updateAvailableItem.Hide()

	systray.AddSeparator()

	toggleUIItem := systray.AddMenuItem("Toggle UI", "Show or hide the SISR UI")
	enableOverlayItem := systray.AddMenuItemCheckbox(
		"Enable Steam Overlay",
		"Enables the Steam overlay as transparent borderless window",
		c.Config.Fullscreen,
	)

	systray.AddSeparator()

	allowDesktopLayoutItem := systray.AddMenuItemCheckbox(
		"Allow Steam Desktop Layout",
		"Uses Steams Desktop Layout instead of SISR Marker (or specific SISR layout when launched via Steam)",
		c.Config.AllowSteamDesktopLayout,
	)
	openConfiguratorItem := systray.AddMenuItem(
		"Open Steam Input Layout Configurator",
		"Opens the Steam Input Layout configurator",
	)

	systray.AddSeparator()

	exitItem := systray.AddMenuItem("Quit", "Exit SISR")

	t := &tray{
		SISRContext: c,
		notifyCh:    trayNotifyCh,

		updateAvailableItem: updateAvailableItem,

		toggleUIItem:      toggleUIItem,
		enableOverlayItem: enableOverlayItem,

		allowDesktopLayoutItem: allowDesktopLayoutItem,
		openConfiguratorItem:   openConfiguratorItem,

		exitItem: exitItem,
	}

	go t.run(ctx)
}

type tray struct {
	*cmd.SISRContext

	notifyCh TrayNotifyCh

	updateAvailableItem *systray.MenuItem

	toggleUIItem      *systray.MenuItem
	enableOverlayItem *systray.MenuItem

	allowDesktopLayoutItem *systray.MenuItem
	openConfiguratorItem   *systray.MenuItem

	exitItem *systray.MenuItem
}

func (t *tray) run(ctx context.Context) {
	for {
		select {
		case <-ctx.Done():
			systray.Quit()
			return
		case notification := <-t.notifyCh:
			switch n := notification.(type) {
			case *UpdateAvailableNotification:
				t.handleUpdateAvailableNotification(n)
			}
		case <-t.updateAvailableItem.ClickedCh:
			t.handleUpdateAvailableClick(ctx)
		case <-t.toggleUIItem.ClickedCh:
			t.handleToggleUI(ctx)
		case <-t.enableOverlayItem.ClickedCh:
			t.handleToggleOverlay(ctx)
		case <-t.allowDesktopLayoutItem.ClickedCh:
			t.handleAllowDesktopConfig()
		case <-t.openConfiguratorItem.ClickedCh:
			t.handleOpenConfigurator()
		case <-t.exitItem.ClickedCh:
			t.QuitFn()
		}
	}
}

func (t *tray) handleUpdateAvailableNotification(n *UpdateAvailableNotification) {
	toggleText := fmt.Sprintf("⬆️ Update available: %s", n.Version)
	t.updateAvailableItem.SetTitle(toggleText)
	t.updateAvailableItem.SetTooltip(toggleText)
	t.updateAvailableItem.Show()
}

func (t *tray) handleUpdateAvailableClick(ctx context.Context) {
	t.UpdateChecker.SetDismissed(false)
	_, err := cmd.ScheduleWindowDispatch(ctx, t.WindowDispatcher, func(w *sdl.Window, wv webview.WebView) bool {
		w.ShowWindow()
		wv.Eval("window.invalidateAll();")
		_ = t.WindowDispatcher.Schedule(func(w *sdl.Window, wv webview.WebView) any {
			wv.SetVisible(true)
			return nil
		})
		err := extras.SetCursorHitTest(w, true)
		if err != nil {
			slog.Error("Failed setting window cursor hittest", "error", err)
		}
		return true
	})
	if err != nil {
		slog.Error("Failed to dispatch update available click to window", "error", err)
	}
}

func (t *tray) handleToggleUI(ctx context.Context) {
	_, err := cmd.ScheduleWindowDispatch(ctx, t.WindowDispatcher, func(w *sdl.Window, wv webview.WebView) bool {
		t.Config.Lock()
		fullscreen := t.Config.Fullscreen
		kbmEnabled := t.Config.KeyboardMouseEmulation
		t.Config.Unlock()
		windowHidden := w.GetWindowFlags()&sdl.WindowFlagHidden != 0
		uiVisible := wv.Visible() && !windowHidden
		if uiVisible {
			if !kbmEnabled {
				err := extras.SetCursorHitTest(w, false)
				if err != nil {
					slog.Error("Failed setting window cursor hittest", "error", err)
				}
			}
			if !fullscreen {
				w.HideWindow()
			}
			wv.SetVisible(false)
			return false
		} else {
			w.ShowWindow()
			wv.Eval("window.invalidateAll();")
			_ = t.WindowDispatcher.Schedule(func(w *sdl.Window, wv webview.WebView) any {
				wv.SetVisible(true)
				return nil
			})
			err := extras.SetCursorHitTest(w, true)
			if err != nil {
				slog.Error("Failed setting window cursor hittest", "error", err)
			}
			return true
		}
	})
	if err != nil {
		slog.Error("Failed to toggle UI visibility", "error", err)
	}
}

func (t *tray) handleToggleOverlay(ctx context.Context) {
	if t.enableOverlayItem.Checked() {
		t.enableOverlayItem.Uncheck()
	} else {
		t.enableOverlayItem.Check()
	}
	t.Config.Lock()
	t.Config.Fullscreen = t.enableOverlayItem.Checked()
	fullscreen := t.Config.Fullscreen
	kbmEnabled := t.Config.KeyboardMouseEmulation
	t.Config.Unlock()

	err, dispatchErr := cmd.ScheduleWindowDispatch(ctx, t.WindowDispatcher, func(w *sdl.Window, wv webview.WebView) error {
		if wv.Visible() {
			wv.SetVisible(false)
		}
		if fullscreen {
			w.ShowWindow()
			wv.Eval("window.invalidateAll();")
			t.WindowDispatcher.Schedule(func(w *sdl.Window, wv webview.WebView) any {
				var err error
				if !kbmEnabled {
					err = extras.SetCursorHitTest(w, false)
					if err != nil {
						slog.Error("Failed setting window cursor hittest", "error", err)
					}
				}
				err = w.SetWindowFullscreen(true)
				if err != nil {
					slog.Debug("Failed to set window fullscreen", "error", err)
					return err
				}
				err = w.SetWindowAlwaysOnTop(true)
				if err != nil {
					slog.Debug("Failed to set window always on top", "error", err)
					return err
				}
				err = w.SetWindowResizable(false)
				if err != nil {
					slog.Debug("Failed to set window resizable", "error", err)
					return err
				}
				err = w.SetWindowBordered(false)
				if err != nil {
					slog.Debug("Failed to set window bordered", "error", err)
					return err
				}
				return nil
			})
		} else {
			w.HideWindow()
			_ = t.WindowDispatcher.Schedule(func(w *sdl.Window, wv webview.WebView) any {
				var err error
				err = extras.SetCursorHitTest(w, true)
				if err != nil {
					slog.Error("Failed setting window cursor hittest", "error", err)
				}
				if !wv.Visible() {
					wv.SetVisible(true)
				}
				err = w.SetWindowFullscreen(false)
				if err != nil {
					slog.Debug("Failed to set window fullscreen", "error", err)
					return err
				}
				err = w.SetWindowAlwaysOnTop(false)
				if err != nil {
					slog.Debug("Failed to set window always on top", "error", err)
					return err
				}
				err = w.SetWindowResizable(false)
				if err != nil {
					slog.Debug("Failed to set window resizable", "error", err)
					return err
				}
				err = w.SetWindowBordered(true)
				if err != nil {
					slog.Debug("Failed to set window bordered", "error", err)
					return err
				}
				return nil
			})
		}
		return nil
	})
	if dispatchErr != nil {
		slog.Error("Error dispatching fullscreen state change to window", "error", dispatchErr)
	}
	if err != nil {
		slog.Error("Failed to change fullscreen state", "error", err)
	}

}

func (t *tray) handleAllowDesktopConfig() {
	t.Config.Lock()
	noSteam := t.Config.NoSteam
	t.Config.Unlock()
	if noSteam {
		return
	}

	if t.allowDesktopLayoutItem.Checked() {
		t.allowDesktopLayoutItem.Uncheck()
	} else {
		t.allowDesktopLayoutItem.Check()
	}
	if t.allowDesktopLayoutItem.Checked() {
		err := t.BindingEnforcer.ForceInputAppID(0) // 0 reverts to non-forced, including desktop-layout when out of focus
		if err != nil {
			slog.Error("Failed to reset forced inputAppID", "error", err)
			t.allowDesktopLayoutItem.Uncheck()
			return
		}
	} else {
		err := t.BindingEnforcer.ForceOwnAppID()
		if err != nil {
			slog.Error("Failed to force SteamInput layout", "error", err)
			t.allowDesktopLayoutItem.Check()
			return
		}
	}
	t.Config.Lock()
	t.Config.AllowSteamDesktopLayout = t.allowDesktopLayoutItem.Checked()
	t.Config.Unlock()
}

func (t *tray) handleOpenConfigurator() {

	t.Config.Lock()
	steamCfg := *t.Config.Steam
	t.Config.Unlock()

	args := &cefpayloads.OpenConfiguratorArgs{
		AppID: 413080, // Desktop Layout
	}

	if !t.allowDesktopLayoutItem.Checked() {
		appIDStr := os.Getenv("SteamAppId")
		if appIDStr == "" || appIDStr == "0" {
			appIDStr = os.Getenv("SISR_MARKER_ID")
		}
		appID, err := strconv.ParseUint(appIDStr, 10, 32)
		if err != nil {
			slog.Error("failed to convert appID", "error", err)
			return
		}
		if appID != 0 {
			args.AppID = uint32(appID)
		}
	}

	_, err := cefpayloads.NewOpenConfigurator(&steamCfg).Execute(context.Background(), args)
	if err != nil {
		slog.Error("failed to open configurator", "error", err)
	}

}
