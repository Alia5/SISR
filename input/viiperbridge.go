package input

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"slices"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/Alia5/SISR/config"
	"github.com/Alia5/SISR/helper"
	"github.com/Alia5/SISR/input/viiperdevice"
	"github.com/Alia5/SISR/sdl"
	"github.com/Alia5/VIIPER/viiperclient"
	"github.com/Alia5/VIIPER/viipertypes"
)

type ViiperBridge interface {
	CreateDevice(ctx context.Context, gamepadID sdl.GamepadID, deviceType string) (chan *viiperdevice.Device, chan error)
	IsCreateDeviceScheduled(gamepadID sdl.GamepadID) bool
	Ping(ctx context.Context) (*viipertypes.PingResponse, error)
	Info() *viipertypes.PingResponse
	Ready() bool
	ResolvedAddressAndPort() string
	IsLoopbackAddress() bool
}

const minSupportedVIIPERVersion = "v0.6.1"
const expectedServerName = "VIIPER"
const defaultDeviceType = "xbox360"
const defaultAddress = "localhost:3242"
const startupPingTimeout = 1 * time.Second

var ErrInvalidViiperServer = errors.New("invalid VIIPER server")
var ErrUnsupportedViiperVersion = errors.New("unsupported VIIPER version")

func NewViiperBridge(ctx context.Context, dl DeviceStore, cfg *config.Viiper) ViiperBridge {
	address := defaultAddress
	if cfg != nil && cfg.Address != "" {
		address = cfg.Address
	}
	var client *viiperclient.Client
	if cfg != nil && cfg.Password != "" {
		client = viiperclient.NewWithPassword(address, cfg.Password)
	} else {
		client = viiperclient.New(address)
	}
	v := &viiperBridge{
		client:     client,
		deviceList: dl,
		scheduled:  make(map[sdl.GamepadID]*createDeviceEntry),
		cfg:        cfg,
		shutdown:   ctx.Done(),
	}
	go func() {
		ctx, cancel := context.WithTimeout(ctx, startupPingTimeout)
		defer cancel()

		_, err := v.Ping(ctx)
		if err != nil {
			slog.Error("Initial VIIPER ping failed", "error", err)
		}
	}()
	return v
}

type viiperBridge struct {
	client           *viiperclient.Client
	busID            uint32
	viiperServerInfo *viipertypes.PingResponse
	deviceList       DeviceStore
	cfg              *config.Viiper

	scheduled                  map[sdl.GamepadID]*createDeviceEntry
	viiperServerSpawnAttempted bool
	shutdown                   <-chan struct{}

	mtx sync.Mutex
}

type createDeviceEntry struct {
	deviceType   string
	createdChans []chan *viiperdevice.Device
	errorChans   []chan error
}

func (v *viiperBridge) CreateDevice(ctx context.Context, gamepadID sdl.GamepadID, deviceType string) (chan *viiperdevice.Device, chan error) {
	v.mtx.Lock()
	defer v.mtx.Unlock()

	deviceChan := make(chan *viiperdevice.Device, 1)
	errorChan := make(chan error, 1)
	if meh, ok := v.scheduled[gamepadID]; ok {
		meh.deviceType = deviceType
		meh.createdChans = append(meh.createdChans, deviceChan)
		meh.errorChans = append(meh.errorChans, errorChan)
		return deviceChan, errorChan
	} else {
		v.scheduled[gamepadID] = &createDeviceEntry{
			deviceType:   deviceType,
			createdChans: []chan *viiperdevice.Device{deviceChan},
			errorChans:   []chan error{errorChan},
		}
	}
	go func() {
		defer func() {
			v.mtx.Lock()
			defer v.mtx.Unlock()
			delete(v.scheduled, gamepadID)
		}()

		busID, err := v.ensureBus(ctx)
		if err != nil {
			v.mtx.Lock()
			defer v.mtx.Unlock()
			for _, ch := range v.scheduled[gamepadID].errorChans {
				ch <- err
			}
			return
		}

		v.mtx.Lock()
		deviceType := v.scheduled[gamepadID].deviceType
		v.mtx.Unlock()
		if deviceType == "" {
			deviceType = defaultDeviceType
		}

		s, r, err := v.client.AddDeviceAndConnect(ctx, busID, deviceType, nil)
		if err != nil {
			v.mtx.Lock()
			defer v.mtx.Unlock()
			for _, ch := range v.scheduled[gamepadID].errorChans {
				ch <- err
			}
			return
		}
		closeFunc := func() error {
			_, err := v.client.DeviceRemoveCtx(context.Background(), r.BusID, r.DevID)
			return err
		}
		vd := viiperdevice.New(s, r, closeFunc)
		v.mtx.Lock()
		defer v.mtx.Unlock()
		for _, ch := range v.scheduled[gamepadID].createdChans {
			ch <- vd
		}
	}()
	return deviceChan, errorChan
}

func (v *viiperBridge) IsCreateDeviceScheduled(gamepadID sdl.GamepadID) bool {
	v.mtx.Lock()
	defer v.mtx.Unlock()

	_, ok := v.scheduled[gamepadID]
	return ok
}

func (v *viiperBridge) Info() *viipertypes.PingResponse {
	v.mtx.Lock()
	defer v.mtx.Unlock()

	if v.viiperServerInfo != nil {
		vi := *v.viiperServerInfo
		return &vi
	}
	return nil
}

func (v *viiperBridge) Ping(ctx context.Context) (*viipertypes.PingResponse, error) {
	slog.Debug("Pinging VIIPER server...")
	resp, err := v.client.PingCtx(ctx)
	if err != nil {
		v.mtx.Lock()
		v.viiperServerInfo = nil
		if runtime.GOOS == "windows" && !v.viiperServerSpawnAttempted && v.cfg != nil {
			if isLoopbackAddress(v.cfg.Address) {
				v.viiperServerSpawnAttempted = true
				defer v.trySpawnViiperServer()
			}
		}
		v.mtx.Unlock()
		return nil, err
	}
	if resp.Server != expectedServerName {
		return nil, fmt.Errorf("%w: got %q, expected %q", ErrInvalidViiperServer, resp.Server, expectedServerName)
	}

	v.mtx.Lock()
	defer v.mtx.Unlock()
	v.viiperServerInfo = resp
	slog.Info("VIIPER server ping successful", "version", resp.Version)

	if !isVersionSupported(resp.Version) {
		return nil, fmt.Errorf("%w: got %q, need >= %q", ErrUnsupportedViiperVersion, resp.Version, minSupportedVIIPERVersion)
	}

	return resp, nil
}

func (v *viiperBridge) Ready() bool {
	v.mtx.Lock()
	defer v.mtx.Unlock()

	return v.viiperServerInfo != nil
}

func (v *viiperBridge) ResolvedAddressAndPort() string {
	v.mtx.Lock()
	defer v.mtx.Unlock()

	if v.cfg != nil && v.cfg.Address != "" {
		ip, port, err := lookupIP(v.cfg.Address)
		if err != nil {
			slog.Error("Failed to lookup VIIPER address", "address", v.cfg.Address, "error", err)
			slog.Info("Returning as is...")
			return v.cfg.Address
		}
		return fmt.Sprintf("%s:%s", ip.String(), port)
	}
	return defaultAddress
}

func (v *viiperBridge) IsLoopbackAddress() bool {
	v.mtx.Lock()
	defer v.mtx.Unlock()

	address := defaultAddress
	if v.cfg != nil && v.cfg.Address != "" {
		address = v.cfg.Address
	}

	return isLoopbackAddress(address)
}

func (v *viiperBridge) ensureBus(ctx context.Context) (busID uint32, err error) {
	v.mtx.Lock()
	defer v.mtx.Unlock()

	if v.busID == 0 {
		slog.Debug("No previous VIIPER bus used, creating new bus")
		bus, err := v.client.BusCreateCtx(ctx, 0)
		if err != nil {
			return 0, err
		}
		v.busID = bus.BusID
		slog.Info("Created VIIPER bus", "busID", v.busID)
		return v.busID, nil
	}

	busResp, err := v.client.BusListCtx(ctx)
	if err != nil {
		return 0, err
	}
	if !slices.Contains(busResp.Buses, v.busID) {
		slog.Warn("Created VIIPER bus not found; Recreating")
		bus, err := v.client.BusCreateCtx(ctx, v.busID)
		if err != nil {
			return 0, err
		}
		v.busID = bus.BusID
		slog.Info("Re-Created VIIPER bus", "busID", v.busID)
	}
	return v.busID, nil
}

func isVersionSupported(v string) bool {
	parts := strings.Split(v, ".")
	if len(parts) != 3 {
		return false
	}
	major, err := strconv.Atoi(strings.Trim(parts[0], "vV"))
	if err != nil {
		return false
	}
	minor, err := strconv.Atoi(parts[1])
	if err != nil {
		return false
	}
	patchPart := parts[2]
	for i, r := range patchPart {
		if r < '0' || r > '9' {
			patchPart = patchPart[:i]
			break
		}
	}
	if patchPart == "" {
		return false
	}
	patch, err := strconv.Atoi(patchPart)
	if err != nil {
		return false
	}

	minParts := strings.Split(minSupportedVIIPERVersion, ".")
	minMajor, _ := strconv.Atoi(strings.Trim(minParts[0], "vV"))
	minMinor, _ := strconv.Atoi(minParts[1])
	minPatch, _ := strconv.Atoi(minParts[2])

	if major < minMajor {
		return false
	}
	if major == minMajor && minor < minMinor {
		return false
	}
	if major == minMajor && minor == minMinor && patch < minPatch {
		return false
	}
	return true
}

func lookupIP(addr string) (net.IP, string, error) {
	host, port, splitErr := net.SplitHostPort(addr)
	if splitErr != nil {
		slog.Error(
			"Failed to split VIIPER address",
			"address", addr,
			"error", splitErr,
		)
		return nil, port, splitErr
	}
	ips, lookupErr := net.LookupIP(host)
	if lookupErr != nil {
		slog.Error(
			"Failed to lookup VIIPER address host",
			"host", host,
			"error", lookupErr,
		)
		return nil, port, lookupErr
	}
	return ips[0], port, nil
}

func isLoopbackAddress(addr string) bool {
	ip, _, err := lookupIP(addr)
	if err != nil {
		return false
	}
	return ip.IsLoopback()
}

func (v *viiperBridge) trySpawnViiperServer() {
	slog.Debug("Attempting to spawn bundled VIIPER")
	ownExecutable, err := helper.GetOwnExecutablePath()
	if err != nil {
		slog.Error("viiper_spawn: couldn't detect own executable", "error", err)
	}
	viiperPath := filepath.Join(filepath.Dir(ownExecutable), "viiper.exe")
	if _, err := os.Stat(viiperPath); os.IsNotExist(err) {
		slog.Error("viiper_spawn: viiper.exe not found next to SISR executable", "path", viiperPath)
		return
	}

	cmd := exec.Command(viiperPath, "server", "--log.file=viiper.log")
	err = cmd.Start()
	if err != nil {
		slog.Error("viiper_spawn: failed to start VIIPER server process", "error", err)
		return
	}
	go func() {
		_ = cmd.Wait()
	}()
	slog.Info("viiper_spawn: VIIPER server process started", "pid", cmd.Process.Pid)
	go func() {
		for range 10 {
			time.Sleep(1 * time.Second)
			ctx, cancel := context.WithTimeout(context.Background(), startupPingTimeout)
			r, err := v.Ping(ctx)
			if err == nil {
				slog.Info("viiper_spawn: successfully connected to VIIPER server", "version", r.Version)
				cancel()
				if v.shutdown != nil {
					<-v.shutdown
					err = cmd.Process.Kill()
					if err != nil && !errors.Is(err, os.ErrProcessDone) {
						slog.Error("viiper_spawn: failed to stop VIIPER server process", "pid", cmd.Process.Pid, "error", err)
						return
					}
					slog.Info("viiper_spawn: stopped VIIPER server process", "pid", cmd.Process.Pid)
				}

				return
			}
			cancel()
		}
	}()
}
