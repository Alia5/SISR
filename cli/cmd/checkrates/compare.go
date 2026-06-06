package checkrates

import (
	"context"
	"fmt"
	"log/slog"
	"strings"
	"time"

	"github.com/Alia5/SISR/sdl"
)

type comparePair struct {
	realName      string
	virtName      string
	realIntervals []float64
	virtIntervals []float64
	latencies     []float64
}

type pairState struct {
	cp        *comparePair
	realID    int32
	virtualID int32

	activeReal    bool
	activeVirt    bool
	lastRealStamp uint64
	lastVirtStamp uint64
	lastRealSnap  gamepadState
	lastVirtSnap  gamepadState

	latencyAnchor  uint64
	latencyPending bool
}

func collectCompare(ctx context.Context, renderer sdl.Renderer, stopAfter time.Duration, keepDups bool) []comparePair {
	pads := map[int32]*sdl.Gamepad{}
	padIsVirtual := map[int32]bool{}
	pairByRealID := map[int32]*pairState{}
	pairByVirtID := map[int32]*pairState{}
	openOrder := []int32{}

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
		seen := map[*comparePair]bool{}
		for _, ps := range pairByRealID {
			if seen[ps.cp] {
				continue
			}
			seen[ps.cp] = true
			n += len(ps.cp.realIntervals) + len(ps.cp.virtIntervals)
		}
		return n
	}

	startCollecting := func() {
		if anyActive {
			return
		}
		anyActive = true
		if stopAfter > 0 {
			collectCtx, collectCancel = context.WithTimeout(ctx, stopAfter)
			fmt.Printf("Collecting for %s...", stopAfter)
		} else {
			fmt.Print("Collecting...")
		}
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
				steamHandle := g.GetSteamHandle()
				pads[e.Which] = g

				if steamHandle == 0 {
					padIsVirtual[e.Which] = false
					ps := &pairState{
						cp: &comparePair{
							realName: g.Name(),
						},
						realID:    e.Which,
						virtualID: -1,
					}
					pairByRealID[e.Which] = ps
					openOrder = append(openOrder, e.Which)
					slog.Info("Real controller", "name", g.Name(), "id", e.Which)
				} else {
					padIsVirtual[e.Which] = true

					var paired *pairState
					for i := len(openOrder) - 1; i >= 0; i-- {
						ps, ok := pairByRealID[openOrder[i]]
						if ok && ps.virtualID == -1 {
							paired = ps
							break
						}
					}
					if paired != nil {
						paired.virtualID = e.Which
						paired.cp.virtName = g.Name()
						pairByVirtID[e.Which] = paired
						slog.Info("Steam virtual controller paired",
							"virtual", g.Name(), "virtual_id", e.Which,
							"real", paired.cp.realName, "real_id", paired.realID)
					} else {
						ps := &pairState{
							cp: &comparePair{
								realName: g.Name(),
								virtName: g.Name(),
							},
							realID:    e.Which,
							virtualID: e.Which,
						}
						pairByRealID[e.Which] = ps
						pairByVirtID[e.Which] = ps
						openOrder = append(openOrder, e.Which)
						slog.Info("Single-pad controller", "name", g.Name(), "id", e.Which)
					}
				}

			case sdl.EventTypeGamepadRemoved:
				if g, ok := pads[e.Which]; ok {
					g.Close()
					delete(pads, e.Which)
				}

			case sdl.EventTypeGamepadUpdateComplete:
				g, ok := pads[e.Which]
				if !ok {
					return
				}
				stamp := e.Timestamp
				isVirt := padIsVirtual[e.Which]

				if !isVirt {
					handleRealUpdate(g, e.Which, stamp, keepDups, pairByRealID, startCollecting, totalCollected)
				} else {
					handleVirtUpdate(g, e.Which, stamp, keepDups, pairByVirtID, startCollecting, totalCollected)
				}
			}

		case *sdl.QuitEvent:
			stop = true
		}
		return
	}

	finish := func() []comparePair {
		if collectCancel != nil {
			collectCancel()
		}
		fmt.Println()
		seen := map[*comparePair]bool{}
		var out []comparePair
		for _, id := range openOrder {
			ps, ok := pairByRealID[id]
			if !ok || seen[ps.cp] {
				continue
			}
			seen[ps.cp] = true
			out = append(out, *ps.cp)
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

func stateChangeSummary(old, cur gamepadState) string {
	var parts []string
	for i := range trackedAxes {
		if old.axes[i] != cur.axes[i] {
			parts = append(parts, fmt.Sprintf("%s:%d->%d", axisNames[i], old.axes[i], cur.axes[i]))
		}
	}
	for i := range buttonNames {
		oldPressed := (old.buttons>>uint(i))&1 == 1
		newPressed := (cur.buttons>>uint(i))&1 == 1
		if oldPressed != newPressed {
			arrow := " pressed"
			if !newPressed {
				arrow = " released"
			}
			parts = append(parts, buttonNames[i]+arrow)
		}
	}
	for i := range old.touchpads {
		if old.touchpads[i] != cur.touchpads[i] {
			t := cur.touchpads[i]
			if t.down {
				parts = append(parts, fmt.Sprintf("touch[%d]:(%.3f,%.3f)", i, t.x, t.y))
			} else {
				parts = append(parts, fmt.Sprintf("touch[%d]:up", i))
			}
		}
	}
	return strings.Join(parts, " ")
}

func handleRealUpdate(
	g *sdl.Gamepad,
	id int32,
	stamp uint64,
	keepDups bool,
	pairByRealID map[int32]*pairState,
	startCollecting func(),
	totalCollected func() int,
) {
	ps, ok := pairByRealID[id]
	if !ok {
		return
	}

	if !ps.activeReal {
		if !isActive(g) {
			return
		}
		ps.activeReal = true
		ps.lastRealStamp = stamp
		ps.lastRealSnap = getSnapshot(g)
		startCollecting()
		return
	}

	snap := getSnapshot(g)
	if !keepDups && snap == ps.lastRealSnap {
		ps.lastRealStamp = stamp
		return
	}
	slog.Debug("real changed", "id", id, "name", ps.cp.realName, "ts_ns", stamp, "changes", stateChangeSummary(ps.lastRealSnap, snap))
	ps.lastRealSnap = snap

	ms := float64(stamp-ps.lastRealStamp) / 1_000_000.0
	if ms > 0.05 && ms < 200 {
		ps.cp.realIntervals = append(ps.cp.realIntervals, ms)
		fmt.Printf("\r  %d intervals collected", totalCollected())
	}
	ps.lastRealStamp = stamp
	if !ps.latencyPending {
		ps.latencyAnchor = stamp
		ps.latencyPending = true
	}
}

func handleVirtUpdate(
	g *sdl.Gamepad,
	id int32,
	stamp uint64,
	keepDups bool,
	pairByVirtID map[int32]*pairState,
	startCollecting func(),
	totalCollected func() int,
) {
	ps, ok := pairByVirtID[id]
	if !ok {
		return
	}

	if !ps.activeVirt {
		if !isActive(g) {
			return
		}
		ps.activeVirt = true
		ps.lastVirtStamp = stamp
		ps.lastVirtSnap = getSnapshot(g)
		startCollecting()
		return
	}

	snap := getSnapshot(g)
	if !keepDups && snap == ps.lastVirtSnap {
		ps.lastVirtStamp = stamp
		return
	}
	slog.Debug("virtual changed", "id", id, "name", ps.cp.virtName, "ts_ns", stamp, "changes", stateChangeSummary(ps.lastVirtSnap, snap))
	ps.lastVirtSnap = snap

	ms := float64(stamp-ps.lastVirtStamp) / 1_000_000.0
	if ms > 0.05 && ms < 200 {
		ps.cp.virtIntervals = append(ps.cp.virtIntervals, ms)
		fmt.Printf("\r  %d intervals collected", totalCollected())
	}
	ps.lastVirtStamp = stamp

	if ps.latencyPending {
		lat := float64(stamp-ps.latencyAnchor) / 1_000_000.0
		if lat >= 0 && lat < 50 {
			ps.cp.latencies = append(ps.cp.latencies, lat)
			slog.Debug("latency measured", "real_id", ps.realID, "virt_id", id, "lat_ms", lat, "trigger", stateChangeSummary(ps.lastVirtSnap, snap))
		}
		ps.latencyPending = false
	}
}
