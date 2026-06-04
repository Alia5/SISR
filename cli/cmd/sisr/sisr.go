package sisr

import (
	"context"
	"log/slog"
	"os"
	"os/signal"
	"path/filepath"
	"syscall"
	"time"

	"github.com/Alia5/SISR/cefpayloads"
	"github.com/Alia5/SISR/cmd"
	"github.com/Alia5/SISR/config"
	"github.com/Alia5/SISR/event"
	"github.com/Alia5/SISR/helper"
	"github.com/Alia5/SISR/input"
	"github.com/Alia5/SISR/input/steaminputbindings"
	"github.com/Alia5/SISR/sdl"
	"github.com/Alia5/SISR/sdl/extras"
	"github.com/Alia5/SISR/steam"
	"github.com/Alia5/SISR/tray"
	"github.com/Alia5/SISR/update"
	"github.com/Alia5/SISR/webview"
)

type SISR struct {
	config.AutoUpdate          `embed:""`
	config.RunMisc             `embed:""`
	config.ControllerEmulation `embed:""`
	config.KbMEmuation         `embed:""`
	config.API                 `embed:"" prefix:"api."`
	config.Viiper              `embed:"" prefix:"viiper."`
	config.Window              `embed:"" prefix:"window."`
	config.Steam               `embed:"" prefix:"steam."`

	//
	lastRenderTime      time.Time
	targetFrameDuration time.Duration
}

func (s *SISR) Run(cfg config.Global) error {
	ctx, stop := signal.NotifyContext(
		context.Background(),
		os.Interrupt, syscall.SIGTERM,
	)
	defer stop()
	defer s.cleanup(&s.Steam)

	if s.MaxFPS == 0 {
		s.targetFrameDuration = 0
	} else {
		s.targetFrameDuration = time.Second / 60
	}

	setSDLHintEnv()
	setSDLHints()
	s.checkInitialLaunch()

	launchedViaSteam, launchedInGameMode := steam.LaunchedViaSteam()
	_ = launchedInGameMode

	if !launchedViaSteam {
		slog.Info("Not launched via Steam, setting env...")
		err := steam.SetMarkerEnv(s.Steam.InstallDir, s.Steam.UserID) //nolint:staticcheck
		if err != nil {
			slog.Error("Failed to set Steam marker environment", "error", err)
		}
		slog.Info("Loading overlay...")
		err = steam.LoadOverlay(s.Steam.InstallDir) //nolint:staticcheck
		if err != nil {
			slog.Error("Failed to load Steam overlay", "error", err)
		}
	} else {
		slog.Info("Launched via Steam")
	}

	window, renderer, wv, err := s.createWindow(&s.Window)
	if err != nil {
		return err
	}
	defer func() {
		wv.Destroy()
		renderer.Destroy()
		window.Destroy()
	}()

	unhookSteamHid()
	err = sdl.InitSubSystem(sdl.InitFlagGamepad)
	if err != nil {
		slog.Error("Failed to initialize SDL Gamepad subsystem", "error", err)
		return err
	}
	err = sdl.InitSubSystem(sdl.InitFlagSensor)
	if err != nil {
		slog.Error("Failed to initialize SDL Sensor subsystem", "error", err)
		return err
	}
	err = sdl.InitSubSystem(sdl.InitFlagHaptic)
	if err != nil {
		slog.Error("Failed to initialize SDL Haptic subsystem", "error", err)
		return err
	}

	trayNotifyCh := make(chan any, 10)
	winDispatcher := cmd.NewWindowDispatcher[any]()

	bindingEnforcer := steaminputbindings.NewEnforcer()
	eventRouter := event.NewRouter()
	deviceStore, deviceStoreClose, err := input.NewDeviceStore(s.NoSteam, s.GyroPassthrough)
	if err != nil {
		slog.Error("Failed to initialite DeviceStore", "error", err)
	}
	defer deviceStoreClose()
	viiperBridge := input.NewViiperBridge(ctx, deviceStore, &s.Viiper)
	if s.KeyboardMouseEmulation && viiperBridge.IsLoopbackAddress() {
		slog.Warn("Keyboard/mouse emulation requires non-loopback VIIPER address; disabling", "viiperAddress", viiperBridge.ResolvedAddressAndPort())
		s.KeyboardMouseEmulation = false
	}
	updateChecker := update.NewChecker(
		s.UpdateNotify,
		func() {
			winDispatcher.Schedule(func(w *sdl.Window, wv webview.WebView) any {
				slog.Debug("UpdateNotify: Showing Window + WebView")
				w.ShowWindow()
				err := extras.SetCursorHitTest(w, true)
				if err != nil {
					slog.Error("Failed setting window cursor hittest", "error", err)
				}
				wv.Eval("window.invalidateAll();")
				winDispatcher.Schedule(func(w *sdl.Window, wv webview.WebView) any {
					wv.SetVisible(true)
					return nil
				})
				return nil
			})
		},
		func(version string) {
			slog.Debug("Notifying tray about update availability", "version", version)
			trayNotifyCh <- &tray.UpdateAvailableNotification{
				Version: version,
			}
		},
	)

	cmdCtx := &cmd.SISRContext{
		WindowDispatcher: winDispatcher,
		DeviceStore:      deviceStore,
		ViiperBridge:     viiperBridge,
		BindingEnforcer:  bindingEnforcer,
		QuitFn:           stop,
		UpdateChecker:    updateChecker,
		Config: &cmd.SessionConfig{
			AutoUpdate:          &s.AutoUpdate,
			RunMisc:             &s.RunMisc,
			ControllerEmulation: &s.ControllerEmulation,
			KbMEmuation:         &s.KbMEmuation,
			Viiper:              &s.Viiper,
			Window:              &s.Window,
			Steam:               &s.Steam,
		},
	}

	registerEventHandlers(eventRouter, cmdCtx, window, wv)

	_, apiAddr := s.runAPIServer(cmdCtx)
	frontendAddr := s.FrontendAddress
	if frontendAddr == "" {
		frontendAddr = apiAddr
	}
	cmdCtx.APIAddr = &apiAddr

	wv.Navigate(frontendAddr)

	if !s.AllowSteamDesktopLayout {
		go func() {
			// HACK: attempt to workaround possible bug in Steam where inputs might get stuck
			time.Sleep(2 * time.Second)

			err = bindingEnforcer.ForceOwnAppID()
			if err != nil {
				slog.Error("Failed to force SteamInput layout", "error", err)
			}
		}()
	}

	tray.Run(ctx, cmdCtx, trayNotifyCh)

	go func() {
		ctx, cancel := context.WithTimeout(ctx, 10*time.Second)
		defer cancel()
		_, err := updateChecker.CheckForUpdate(ctx)
		if err != nil {
			slog.Error("Failed to check for updates", "error", err)
		}
	}()

	if !s.Window.Show { // nolint
		go func() {
			winDispatcher.Schedule(func(w *sdl.Window, wv webview.WebView) any {
				if !s.Fullscreen {
					w.HideWindow()
				} else {
					if s.KeyboardMouseEmulation {
						err := extras.SetCursorHitTest(w, true)
						if err != nil {
							slog.Error("Failed setting window cursor hittest", "error", err)
						}
					} else {
						err := extras.SetCursorHitTest(w, false)
						if err != nil {
							slog.Error("Failed setting window cursor hittest", "error", err)
						}
					}
				}
				wv.SetVisible(false)
				return nil
			})
		}()
	}

	return s.run(ctx, renderer, window, wv, eventRouter, winDispatcher)
}

func (s *SISR) run(
	ctx context.Context,
	renderer sdl.Renderer,
	window *sdl.Window,
	wv webview.WebView,
	router event.Router,
	dispatcher cmd.WindowDispatcher[any],
) error {
	for {
		select {
		case <-ctx.Done():
			return nil
		default:
		}
		ev, _ := sdl.WaitEventTimeout(s.targetFrameDuration)
		if ev != nil {
			router.RouteEvent(ctx, ev)
			drainEvents(ctx, router)
		}
		dispatcher.Dispatch(window, wv)
		if time.Since(s.lastRenderTime) >= s.targetFrameDuration || ev == nil {
			s.lastRenderTime = time.Now()
			wv.Tick()
			err := renderer.RenderClear()
			if err != nil {
				slog.Error("Failed to clear renderer", "error", err)
				return err
			}
			err = renderer.RenderPresent()
			if err != nil {
				slog.Error("Failed to present renderer", "error", err)
				return err
			}
		}

	}
}

func drainEvents(ctx context.Context, router event.Router) {
	for {
		ev, _ := sdl.PollEvent()
		if ev == nil {
			break
		}
		router.RouteEvent(ctx, ev)
	}
}

func (s *SISR) cleanup(c *config.Steam) {
	slog.Info("Shutting down")

	if s.NoSteam {
		return
	}

	slog.Info("Cleaning up CEF payloads...")
	err := cefpayloads.SISRCleanup(context.Background(), c)
	if err != nil {
		slog.Error("Failed to cleanup CEF payloads", "error", err)
	}
	err = helper.OpenURL("steam://forceinputappid/0")
	if err != nil {
		slog.Error("Failed to reset steam controller config", "error", err)
	}
}

func (s *SISR) checkInitialLaunch() {
	if s.InitialLaunch || s.NoSteam {
		// is ignored / irrelevant
		return
	}
	ownExecutable, err := helper.GetOwnExecutablePath()
	if err != nil {
		slog.Error("Failed to get own executable path", "error", err)
		return
	}
	markerPath := filepath.Join(filepath.Dir(ownExecutable), ".initial_setup_done")
	if _, err := os.Stat(markerPath); os.IsNotExist(err) {
		s.InitialLaunch = true
	}
}
