package main

import (
	"context"
	"log/slog"
	"os"
	"os/signal"
	"runtime"
	"strings"
	"syscall"

	"github.com/Alia5/SISR/internal/loop"
	"github.com/Alia5/SISR/internal/tray"
	"github.com/Alia5/SISR/internal/viiper"
	"github.com/Alia5/SISR/internal/windowutil"

	"github.com/Zyko0/go-sdl3/bin/binsdl"
	"github.com/Zyko0/go-sdl3/sdl"
	"github.com/alecthomas/kong"
	kongtoml "github.com/alecthomas/kong-toml"
	kongyaml "github.com/alecthomas/kong-yaml"
)

type sisr struct {
	Window struct {
		Create     bool `help:"Create a transparent SDL window. (default: false)" default:"false" env:"SISR_CREATE_WINDOW"`
		Fullscreen bool `help:"Start the window in fullscreen mode. (needs createWindow) (default: true)" default:"true" env:"SISR_FULLSCREEN"`
	} `embed:"" prefix:"window."`
	Viiper struct {
		Addr string `help:"VIIPER API-server address (default: localhost:3242)" default:"localhost:3242" env:"SISR_VIIPER_ADDR"`
	} `embed:"" prefix:"viiper."`
	Tray         bool `help:"Show system tray icon (default: true)" default:"true" env:"SISR_TRAY"`
	platformOpts `embed:""`
}

type logHandler struct {
	stdout, stderr slog.Handler
}

func (h logHandler) Enabled(ctx context.Context, level slog.Level) bool {
	if level >= slog.LevelError {
		return h.stderr.Enabled(ctx, level)
	}
	return h.stdout.Enabled(ctx, level)
}
func (h logHandler) Handle(ctx context.Context, r slog.Record) error {
	if r.Level >= slog.LevelError {
		return h.stderr.Handle(ctx, r)
	}
	return h.stdout.Handle(ctx, r)
}
func (h logHandler) WithAttrs(attrs []slog.Attr) slog.Handler {
	return logHandler{stdout: h.stdout.WithAttrs(attrs), stderr: h.stderr.WithAttrs(attrs)}
}
func (h logHandler) WithGroup(name string) slog.Handler {
	return logHandler{stdout: h.stdout.WithGroup(name), stderr: h.stderr.WithGroup(name)}
}

func main() {
	slog.SetDefault(slog.New(logHandler{
		stdout: slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{Level: slog.LevelDebug}),
		stderr: slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{Level: slog.LevelError}),
	}))
	logger := slog.Default()

	defer binsdl.Load().Unload()
	defer sdl.Quit()

	sdl.SetHint("SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS", "1")

	if err := sdl.Init(sdl.INIT_VIDEO | sdl.INIT_GAMEPAD); err != nil {
		panic(err)
	}

	userCfg := findUserConfig(os.Args[1:])
	cfgPaths := struct {
		json []string
		yaml []string
		toml []string
	}{
		json: []string{"./sisr.json"},
		yaml: []string{"./sisr.yaml"},
		toml: []string{"./sisr.toml"},
	}
	if userCfg != "" {
		if strings.HasSuffix(userCfg, ".json") {
			cfgPaths.json = []string{userCfg}
		} else if strings.HasSuffix(userCfg, ".yaml") || strings.HasSuffix(userCfg, ".yml") {
			cfgPaths.yaml = []string{userCfg}
		} else if strings.HasSuffix(userCfg, ".toml") {
			cfgPaths.toml = []string{userCfg}
		}
	}

	var cli sisr
	ctx := kong.Parse(&cli,
		kong.Name("SISR"),
		kong.Description("Steam Input System Redirector"),
		kong.UsageOnError(),
		kong.Configuration(kong.JSON, cfgPaths.json...),
		kong.Configuration(kongyaml.Loader, cfgPaths.yaml...),
		kong.Configuration(kongtoml.Loader, cfgPaths.toml...),
	)
	ctx.Bind(logger)
	err := ctx.Run()
	ctx.FatalIfErrorf(err)
}

func (c *sisr) Run(logger *slog.Logger) error {

	notifyCtx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	var renderer *sdl.Renderer
	var window *sdl.Window

	if runtime.GOOS == "windows" && c.platformOpts.Console {
		if err := showConsole(); err != nil {
			logger.Error("Failed to alloc console", "error", err)
		}
		// Recreate logger with rebound streams
		slog.SetDefault(slog.New(logHandler{
			stdout: slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{Level: slog.LevelDebug}),
			stderr: slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{Level: slog.LevelError}),
		}))
		logger = slog.Default()
	}

	trayChann := tray.Setup(c.Tray)

	if c.Window.Create {
		w, r, err := createWindow(logger, c.Window.Fullscreen)
		if err != nil {
			return err
		}
		window = w
		renderer = r
		defer renderer.Destroy()
		defer window.Destroy()
	}

	var vc *viiper.Client
	vc, err := viiper.NewClient(notifyCtx, c.Viiper.Addr)
	if err != nil {
		return err
	}
	defer vc.Close()

	loop := loop.New(notifyCtx, trayChann, vc)
	err = loop.Run(window, renderer)
	if err != nil {
		if err == sdl.EndLoop {
			return nil
		}
		return err
	}
	return nil
}

func findUserConfig(args []string) string {
	for i := range len(args) {
		a := args[i]
		if strings.HasPrefix(a, "--config=") {
			return a[len("--config="):]
		}
		if a == "--config" && i+1 < len(args) {
			return args[i+1]
		}
	}
	if v := os.Getenv("SISR_CONFIG"); v != "" {
		return v
	}
	return ""
}

func createWindow(logger *slog.Logger, fullscreen bool) (*sdl.Window, *sdl.Renderer, error) {

	windowFlags := sdl.WINDOW_TRANSPARENT |
		sdl.WINDOW_SURFACE_VSYNC_DISABLED |
		sdl.WINDOW_VULKAN
	if fullscreen {
		if runtime.GOOS == "windows" {
			windowFlags |= sdl.WINDOW_BORDERLESS | sdl.WINDOW_ALWAYS_ON_TOP | sdl.WINDOW_NOT_FOCUSABLE
		} else {
			logger.Warn("Fullscreen not currently supported on non windows platforms")
		}
	} else {
		windowFlags |= sdl.WINDOW_RESIZABLE
	}

	window, renderer, err := sdl.CreateWindowAndRenderer(
		"SISR",
		// TODO: auto set resolution
		1280, 720,
		windowFlags,
	)
	if err != nil {
		panic(err)
	}
	if fullscreen && runtime.GOOS == "windows" {
		err := windowutil.MakeClickthrough(window)
		if err != nil {
			logger.Error("Failed to make window clickthrough", "error", err)
		}
	}
	renderer.SetDrawColor(0, 0, 0, 0)

	return window, renderer, nil
}
