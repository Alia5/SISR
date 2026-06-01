package devices

import (
	"context"
	"fmt"
	"net/http"
	"slices"

	"github.com/Alia5/SISR/cmd"
	"github.com/Alia5/SISR/sdl"
	"github.com/Alia5/VIIPER/viipertypes"
	"github.com/danielgtaylor/huma/v2"
)

type DeviceInfoResponse struct {
	Body DeviceInfo
}

type DeviceInfo []*APIDevice

type APIDevice struct {
	RealDevice         *APIGamepad         `json:"real_device,omitempty"`
	SteamVirtualDevice *APIGamepad         `json:"steam_virtual_device,omitempty"`
	VIIPERDevice       *viipertypes.Device `json:"viiper_device,omitempty"`
}

type APIGamepad struct {
	ID          sdl.GamepadID `json:"id"`
	Name        string        `json:"name"`
	SteamHandle string        `json:"steam_handle,omitempty"` //js may overflow uint64...
	Type        string        `json:"type,omitempty"`
	RealType    string        `json:"real_type,omitempty"`
	PlayerIndex int           `json:"player_index"`
	Path        string        `json:"path,omitempty"`
	Serial      string        `json:"serial,omitempty"`
	// TODO: add more fields as required.
}

func Register(a huma.API, c *cmd.SISRContext) {
	huma.Register(a, huma.Operation{
		Method: http.MethodGet,
		Path:   "/api/v1/devices",
	}, deviceList(c))
}

func deviceList(c *cmd.SISRContext) func(ctx context.Context, req *struct{}) (*DeviceInfoResponse, error) {
	return func(ctx context.Context, req *struct{}) (*DeviceInfoResponse, error) {

		devices := c.DeviceStore.Devices()
		apiDevices := make([]*APIDevice, len(devices))
		for i, dev := range devices {
			dev.Lock()
			apiDevices[i] = &APIDevice{}
			if dev.RealGamepad != nil {
				apiDevices[i].RealDevice = gamepadToAPIType(dev.RealGamepad)
			}
			if dev.SteamVirtualGamepad != nil {
				apiDevices[i].SteamVirtualDevice = gamepadToAPIType(dev.SteamVirtualGamepad)
				apiDevices[i].SteamVirtualDevice.SteamHandle = fmt.Sprintf(
					"%v", dev.SteamVirtualGamepad.GetSteamHandle(),
				)
			}
			if dev.ViiperDevice != nil && !dev.ViiperDevice.IsClosed() {
				apiDevices[i].VIIPERDevice = new(dev.ViiperDevice.Info())
			}
			dev.Unlock()
		}

		slices.SortFunc(apiDevices, func(a, b *APIDevice) int {
			idA := 0
			if a.RealDevice != nil {
				idA = int(a.RealDevice.ID)
			}
			if idA == 0 && a.SteamVirtualDevice != nil {
				idA = int(a.SteamVirtualDevice.ID)
			}
			idB := 0
			if b.RealDevice != nil {
				idB = int(b.RealDevice.ID)
			}
			if idB == 0 && b.SteamVirtualDevice != nil {
				idB = int(b.SteamVirtualDevice.ID)
			}
			return idA - idB
		})

		return &DeviceInfoResponse{
			Body: apiDevices,
		}, nil
	}
}

func gamepadToAPIType(gp *sdl.Gamepad) *APIGamepad {
	return &APIGamepad{
		ID:          gp.ID(),
		Name:        gp.Name(),
		Type:        gp.Type().Name(),
		RealType:    gp.RealType().Name(),
		PlayerIndex: gp.GetPlayerIndex(),
		Path:        gp.Path(),
		Serial:      gp.Serial(),
	}
}
