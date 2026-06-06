package checkrates

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/Alia5/SISR/cli/cmd/sisr"
	"github.com/Alia5/SISR/config"
	"github.com/Alia5/SISR/sdl"
	"github.com/Alia5/SISR/steam"
)

const axisThreshold int16 = 3000

type CheckRates struct {
	config.RunMisc `embed:""`
	config.Steam   `embed:"" prefix:"steam."`
	Time           time.Duration `short:"t" default:"5s" help:"Auto-stop after this duration once input is received (e.g. 5s, 500ms)"`
	CompareLatency bool          `aliases:"cl" help:"Unhook Steam HID to expose real controllers and measure added Steam Input latency"`
	KeepDuplicates bool          `aliases:"kd" help:"Keep samples where gamepad state did not change between updates"`
}

func (p *CheckRates) Run(cfg config.Global) error {
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	sisr.SetSDLHintEnv()
	sisr.SetSDLHints()

	launchedViaSteam, _ := steam.LaunchedViaSteam()
	if !launchedViaSteam {
		err := steam.SetMarkerEnv(p.Steam.InstallDir, p.Steam.UserID) //nolint:staticcheck
		if err != nil {
			slog.Warn("Steam marker env", "err", err)
		}
		err = steam.LoadOverlay(p.Steam.InstallDir) //nolint:staticcheck
		if err != nil {
			slog.Warn("Steam overlay", "err", err)
		}
	}

	err := sdl.Init(sdl.InitFlagVideo)
	if err != nil {
		return fmt.Errorf("SDL video: %w", err)
	}

	window, renderer, err := sdl.CreateWindowAndRenderer(
		"SISR Check Rates (Focus this window)",
		1280,
		720,
		sdl.WindowFlagVulkan,
	)
	if err != nil {
		sdl.Quit()
		return fmt.Errorf("window: %w", err)
	}
	defer renderer.Destroy()
	defer window.Destroy()
	_ = renderer.SetRenderDrawColor(0, 0, 0, 0)

	err = sdl.InitSubSystem(sdl.InitFlagGamepad)
	if err != nil {
		return fmt.Errorf("SDL gamepad: %w", err)
	}

	if p.CompareLatency {
		sisr.UnhookSteamHid()
	}

	if p.CompareLatency {
		fmt.Println("Compare-latency mode: measuring real + Steam virtual controllers.")
	}
	if p.KeepDuplicates {
		fmt.Println("Note: keeping samples with unchanged gamepad state.")
	}
	if p.Time > 0 {
		fmt.Printf("Press any button or move any axis to start collecting for %s.\n", p.Time)
	} else {
		fmt.Println("Press any button or move any axis to start collecting. Close the window or Ctrl+C to stop.")
	}
	fmt.Println()

	if p.CompareLatency {
		pairs := collectCompare(ctx, renderer, p.Time, p.KeepDuplicates)
		var valid []comparePair
		for _, cp := range pairs {
			if len(cp.realIntervals) >= 2 || len(cp.virtIntervals) >= 2 {
				valid = append(valid, cp)
			}
		}
		if len(valid) == 0 {
			fmt.Println("Not enough samples collected.")
		} else {
			printCompareResults(valid)
		}
		if launchedViaSteam {
			waitForWindowClose(ctx, renderer)
		}
		return nil
	}

	results := collect(ctx, renderer, p.Time, p.KeepDuplicates)
	var valid []padResult
	for _, r := range results {
		if len(r.intervals) >= 2 {
			valid = append(valid, r)
		}
	}
	if len(valid) == 0 {
		fmt.Println("Not enough samples collected.")
	} else {
		printResults(valid)
	}
	if launchedViaSteam {
		waitForWindowClose(ctx, renderer)
	}
	return nil
}

func waitForWindowClose(ctx context.Context, renderer sdl.Renderer) {
	fmt.Println("Close the window to exit.")
	for {
		select {
		case <-ctx.Done():
			return
		default:
		}

		ev, _ := sdl.WaitEventTimeout(16 * time.Millisecond)
		if ev != nil {
			switch e := ev.(type) {
			case *sdl.QuitEvent:
				return
			case *sdl.WindowEvent:
				if e.Type == sdl.EventTypeWindowCloseRequested {
					return
				}
			}
		} else {
			_ = renderer.RenderClear()
			_ = renderer.RenderPresent()
		}
	}
}
