package checkrates

import (
	"context"
	"fmt"
	"log/slog"
	"slices"
	"time"

	"github.com/Alia5/SISR/sdl"
)

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

type gamepadState struct {
	axes      [6]int16
	buttons   uint32
	touchpads [4]touchpadFingerState
}

func deadzone(v int16) int16 {
	if v > axisThreshold || v < -axisThreshold {
		return v
	}
	return 0
}

func getSnapshot(g *sdl.Gamepad) gamepadState {
	var s gamepadState
	for i, axis := range trackedAxes {
		s.axes[i] = deadzone(g.GetAxis(axis))
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

func isActive(g *sdl.Gamepad) bool {
	for _, axis := range trackedAxes {
		v := g.GetAxis(axis)
		if v > axisThreshold || v < -axisThreshold {
			return true
		}
	}
	if slices.ContainsFunc(trackedButtons, g.GetButton) {
		return true
	}
	numPads := g.NumTouchpads()
	for tp := 0; tp < numPads && tp < 2; tp++ {
		numFingers := g.NumTouchpadFingers(tp)
		for f := 0; f < numFingers && f < 2; f++ {
			ok, down, _, _, _ := g.GetTouchpadFinger(tp, f)
			if ok && down {
				return true
			}
		}
	}
	return false
}

type padResult struct {
	name      string
	intervals []float64
}

func collect(ctx context.Context, renderer sdl.Renderer, stopAfter time.Duration, keepDups bool) []padResult {
	pads := map[int32]*sdl.Gamepad{}
	padNames := map[int32]string{}
	lastStamp := map[int32]uint64{}
	lastSnap := map[int32]gamepadState{}
	active := map[int32]bool{}
	padIntervals := map[int32][]float64{}

	defer func() {
		for _, g := range pads {
			g.Close()
		}
	}()

	anyActive := false
	collectCtx := ctx
	var collectCancel context.CancelFunc

	totalCollected := func() int {
		n := 0
		for _, ivs := range padIntervals {
			n += len(ivs)
		}
		return n
	}

	handle := func(ev sdl.Event) (stop bool) {
		switch e := ev.(type) {
		case *sdl.GamepadDeviceEvent:
			switch e.Type {
			case sdl.EventTypeGamepadAdded:
				g, err := sdl.OpenGamepad(sdl.GamepadID(e.Which))
				if err != nil {
					return
				}
				if g.GetSteamHandle() == 0 {
					g.Close()
					return
				}
				pads[e.Which] = g
				padNames[e.Which] = g.Name()
				padIntervals[e.Which] = nil
				slog.Info("Controller connected", "name", g.Name(), "id", e.Which)

			case sdl.EventTypeGamepadRemoved:
				if g, ok := pads[e.Which]; ok {
					g.Close()
					delete(pads, e.Which)
					delete(lastStamp, e.Which)
					delete(lastSnap, e.Which)
					delete(active, e.Which)
				}

			case sdl.EventTypeGamepadUpdateComplete:
				g, ok := pads[e.Which]
				if !ok {
					return
				}
				stamp := e.Timestamp

				if !active[e.Which] {
					if !isActive(g) {
						return
					}
					active[e.Which] = true
					lastStamp[e.Which] = stamp
					lastSnap[e.Which] = getSnapshot(g)
					if !anyActive {
						anyActive = true
						if stopAfter > 0 {
							collectCtx, collectCancel = context.WithTimeout(ctx, stopAfter)
							fmt.Printf("Collecting for %s...", stopAfter)
						} else {
							fmt.Print("Collecting...")
						}
					}
					return
				}

				snap := getSnapshot(g)
				if !keepDups && snap == lastSnap[e.Which] {
					lastStamp[e.Which] = stamp
					return
				}
				lastSnap[e.Which] = snap

				if prev, ok := lastStamp[e.Which]; ok {
					ms := float64(stamp-prev) / 1_000_000.0
					if ms > 0.05 && ms < 200 {
						padIntervals[e.Which] = append(padIntervals[e.Which], ms)
						fmt.Printf("\r  %d intervals collected", totalCollected())
					}
				}
				lastStamp[e.Which] = stamp
			}

		case *sdl.QuitEvent:
			stop = true
		}
		return
	}

	finish := func() []padResult {
		if collectCancel != nil {
			collectCancel()
		}
		fmt.Println()
		var out []padResult
		for id, ivs := range padIntervals {
			out = append(out, padResult{
				name:      padNames[id],
				intervals: ivs,
			})
		}
		return out
	}

	for {
		select {
		case <-collectCtx.Done():
			return finish()
		default:
		}

		ev, _ := sdl.WaitEventTimeout(16 * time.Millisecond)
		if ev != nil {
			if handle(ev) {
				return finish()
			}
			for {
				ev2, _ := sdl.PollEvent()
				if ev2 == nil {
					break
				}
				if handle(ev2) {
					return finish()
				}
			}
		} else {
			_ = renderer.RenderClear()
			_ = renderer.RenderPresent()
		}
	}
}
