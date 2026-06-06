package monitorinput

import (
	"context"
	"fmt"
	"log/slog"
	"math"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/Alia5/SISR/cli/cmd/sisr"
	"github.com/Alia5/SISR/config"
	"github.com/Alia5/SISR/sdl"
	"github.com/Alia5/SISR/steam"
)

type MonitorInput struct {
	config.RunMisc `embed:""`
	config.Steam   `embed:"" prefix:"steam."`
	Filter         []string `name:"filter"    help:"Only show these input types (repeatable): ls rs lt rt buttons dpad touchpad" sep:","`
	Threshold      float64  `name:"threshold" help:"Minimum change as % of full range to log (0 = any change)" default:"0"`
}

var axisNames = []string{
	"LeftX", "LeftY", "RightX", "RightY", "LeftTrigger", "RightTrigger",
}

var axisFilters = []string{
	"ls", "ls", "rs", "rs", "lt", "rt",
}

var buttonNames = []string{
	"South", "East", "West", "North",
	"Back", "Guide", "Start",
	"LeftStick", "RightStick",
	"LeftShoulder", "RightShoulder",
	"DpadUp", "DpadDown", "DpadLeft", "DpadRight",
}

var buttonFilters = []string{
	"buttons", "buttons", "buttons", "buttons",
	"buttons", "buttons", "buttons",
	"buttons", "buttons",
	"buttons", "buttons",
	"dpad", "dpad", "dpad", "dpad",
}

var trackedAxes = []sdl.GamepadAxis{
	sdl.GamepadAxisLeftX,
	sdl.GamepadAxisLeftY,
	sdl.GamepadAxisRightX,
	sdl.GamepadAxisRightY,
	sdl.GamepadAxisLeftTrigger,
	sdl.GamepadAxisRightTrigger,
}

var trackedButtons = []sdl.GamepadButton{
	sdl.GamepadButtonSouth,
	sdl.GamepadButtonEast,
	sdl.GamepadButtonWest,
	sdl.GamepadButtonNorth,
	sdl.GamepadButtonBack,
	sdl.GamepadButtonGuide,
	sdl.GamepadButtonStart,
	sdl.GamepadButtonLeftStick,
	sdl.GamepadButtonRightStick,
	sdl.GamepadButtonLeftShoulder,
	sdl.GamepadButtonRightShoulder,
	sdl.GamepadButtonDpadUp,
	sdl.GamepadButtonDpadDown,
	sdl.GamepadButtonDpadLeft,
	sdl.GamepadButtonDpadRight,
}

type touchpadFingerState struct {
	x, y, pressure float32
	down           bool
}

type padState struct {
	axes      [6]int16
	buttons   uint32
	touchpads [4]touchpadFingerState
}

func takeSnapshot(g *sdl.Gamepad) padState {
	var s padState
	for i, axis := range trackedAxes {
		s.axes[i] = g.GetAxis(axis)
	}
	for i, btn := range trackedButtons {
		if g.GetButton(btn) {
			s.buttons |= uint32(1) << uint(i)
		}
	}
	idx := 0
	numPads := g.NumTouchpads()
	for tp := 0; tp < numPads && idx < 4; tp++ {
		numFingers := g.NumTouchpadFingers(tp)
		for f := 0; f < numFingers && idx < 4; f++ {
			ok, down, x, y, pressure := g.GetTouchpadFinger(tp, f)
			if ok && down {
				s.touchpads[idx] = touchpadFingerState{
					x:        x,
					y:        y,
					pressure: pressure,
					down:     true,
				}
			}
			idx++
		}
	}
	return s
}

func allowed(filters []string, category string) bool {
	if len(filters) == 0 {
		return true
	}
	for _, f := range filters {
		if f == category {
			return true
		}
	}
	return false
}

func axisExceedsThreshold(old, cur int16, threshold float64) bool {
	if threshold <= 0 {
		return old != cur
	}
	return math.Abs(float64(cur-old))/32767.0*100.0 >= threshold
}

func touchpadExceedsThreshold(old, cur float32, threshold float64) bool {
	if threshold <= 0 {
		return old != cur
	}
	return math.Abs(float64(cur-old))*100.0 >= threshold
}

func logChanges(name string, id int32, stamp uint64, old, cur padState, filters []string, threshold float64) {
	for i := range trackedAxes {
		if !allowed(filters, axisFilters[i]) {
			continue
		}
		if axisExceedsThreshold(old.axes[i], cur.axes[i], threshold) {
			slog.Info("axis",
				"pad", name, "id", id, "ts_ns", stamp,
				"axis", axisNames[i], "value", cur.axes[i],
			)
		}
	}
	for i := range buttonNames {
		if !allowed(filters, buttonFilters[i]) {
			continue
		}
		oldPressed := (old.buttons>>uint(i))&1 == 1
		newPressed := (cur.buttons>>uint(i))&1 == 1
		if oldPressed != newPressed {
			slog.Info("button",
				"pad", name, "id", id, "ts_ns", stamp,
				"button", buttonNames[i], "pressed", newPressed,
			)
		}
	}
	for i := range old.touchpads {
		if !allowed(filters, "touchpad") {
			break
		}
		o := old.touchpads[i]
		c := cur.touchpads[i]
		if o.down != c.down ||
			touchpadExceedsThreshold(o.x, c.x, threshold) ||
			touchpadExceedsThreshold(o.y, c.y, threshold) ||
			touchpadExceedsThreshold(o.pressure, c.pressure, threshold) {
			slog.Info("touchpad",
				"pad", name, "id", id, "ts_ns", stamp,
				"finger", i, "down", c.down, "x", c.x, "y", c.y, "pressure", c.pressure,
			)
		}
	}
}

func (p *MonitorInput) Run(cfg config.Global) error {
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
		"SISR Input Monitor (Focus this window)",
		640,
		480,
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

	slog.Info("Input monitor ready — waiting for gamepads")

	pads := map[int32]*sdl.Gamepad{}
	padNames := map[int32]string{}
	lastSnaps := map[int32]padState{}

	handle := func(ev sdl.Event) (stop bool) {
		switch e := ev.(type) {
		case *sdl.GamepadDeviceEvent:
			switch e.Type {
			case sdl.EventTypeGamepadAdded:
				g, gerr := sdl.OpenGamepad(sdl.GamepadID(e.Which))
				if gerr != nil {
					return
				}
				pads[e.Which] = g
				padNames[e.Which] = g.Name()
				lastSnaps[e.Which] = takeSnapshot(g)
				slog.Info("Gamepad connected",
					"name", g.Name(), "id", e.Which,
					"steam_handle", g.GetSteamHandle(),
				)

			case sdl.EventTypeGamepadRemoved:
				if g, ok := pads[e.Which]; ok {
					slog.Info("Gamepad disconnected", "name", padNames[e.Which], "id", e.Which)
					g.Close()
					delete(pads, e.Which)
					delete(padNames, e.Which)
					delete(lastSnaps, e.Which)
				}

			case sdl.EventTypeGamepadUpdateComplete:
				g, ok := pads[e.Which]
				if !ok {
					return
				}
				cur := takeSnapshot(g)
				logChanges(padNames[e.Which], e.Which, e.Timestamp, lastSnaps[e.Which], cur, p.Filter, p.Threshold)
				lastSnaps[e.Which] = cur
			}

		case *sdl.QuitEvent:
			stop = true
		case *sdl.WindowEvent:
			if e.Type == sdl.EventTypeWindowCloseRequested {
				stop = true
			}
		}
		return
	}

	for {
		select {
		case <-ctx.Done():
			for _, g := range pads {
				g.Close()
			}
			return nil
		default:
		}

		ev, _ := sdl.WaitEventTimeout(16 * time.Millisecond)
		if ev != nil {
			if handle(ev) {
				for _, g := range pads {
					g.Close()
				}
				return nil
			}
			for {
				ev2, _ := sdl.PollEvent()
				if ev2 == nil {
					break
				}
				if handle(ev2) {
					for _, g := range pads {
						g.Close()
					}
					return nil
				}
			}
		} else {
			_ = renderer.RenderClear()
			_ = renderer.RenderPresent()
		}
	}
}
