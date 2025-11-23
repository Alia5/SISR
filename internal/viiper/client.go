package viiper

import (
	"bufio"
	"context"
	"encoding"
	"io"
	"log/slog"

	"github.com/Alia5/VIIPER/apiclient"
	"github.com/Alia5/VIIPER/apitypes"
	"github.com/Alia5/VIIPER/device/xbox360"
	"github.com/Zyko0/go-sdl3/sdl"
)

type Client struct {
	c          *apiclient.Client
	ctx        context.Context
	viiperDevs map[sdl.JoystickID]viiperDevice
	busID      uint32
	createdBus bool
}

func NewClient(ctx context.Context, addr string) (*Client, error) {
	c := apiclient.New(addr)
	busesResp, err := c.BusListCtx(ctx)
	if err != nil {
		return nil, err
	}
	var busID uint32
	createdBus := false
	if len(busesResp.Buses) == 0 {
		r, err := c.BusCreateCtx(ctx, 1)
		if err != nil {
			return nil, err
		}
		busID = r.BusID
		createdBus = true
	} else {
		busID = busesResp.Buses[0]
		for _, b := range busesResp.Buses[1:] {
			if b < busID {
				busID = b
			}
		}
	}
	return &Client{
		ctx:        ctx,
		c:          c,
		viiperDevs: make(map[sdl.JoystickID]viiperDevice),
		busID:      busID,
		createdBus: createdBus,
	}, nil
}

func (vc *Client) AddGamepad(pad *sdl.Gamepad) error {
	if pad == nil {
		return nil
	}
	steamHandle := pad.SteamHandle()
	id, err := pad.ID()
	if err != nil {
		return err
	}
	if _, exists := vc.viiperDevs[id]; exists {
		return nil // already added
	}
	stream, dev, err := vc.c.AddDeviceAndConnect(vc.ctx, vc.busID, "xbox360", nil)
	if err != nil {
		return err
	}
	devCtx, cancel := context.WithCancel(vc.ctx)
	vd := viiperDevice{stream: stream, dev: dev, state: &xbox360.InputState{}, pad: pad, cancel: cancel}
	vc.viiperDevs[id] = vd
	vd.startRumbleReader(devCtx)
	slog.Debug("Added VIIPER device", "id", id, "steamHandle", steamHandle)
	return nil
}

func (vc *Client) RemoveGamepad(id sdl.JoystickID) error {
	dev, ok := vc.viiperDevs[id]
	if !ok {
		return nil
	}
	if dev.cancel != nil {
		dev.cancel()
	}
	_ = dev.Close()
	delete(vc.viiperDevs, id)
	slog.Debug("Removed VIIPER device", "id", id)
	return nil
}

func (vc *Client) UpdateAllGamepads() error {
	for id, dev := range vc.viiperDevs {
		if dev.pad == nil {
			continue
		}
		if err := dev.FromSDLGamepad(dev.pad); err != nil {
			slog.Error("Failed to build state from SDL gamepad", "id", id, "error", err)
			continue
		}
		if err := dev.WriteState(); err != nil {
			slog.Error("Failed to write state to VIIPER", "id", id, "error", err)
			return err
		}
	}
	return nil
}

func (vc *Client) Close() error {
	for _, stream := range vc.viiperDevs {
		stream.Close()
	}
	if vc.createdBus {
		_, _ = vc.c.BusRemoveCtx(vc.ctx, vc.busID)
	}
	return nil
}

type viiperDevice struct {
	stream *apiclient.DeviceStream
	dev    *apitypes.Device
	state  *xbox360.InputState
	pad    *sdl.Gamepad
	cancel context.CancelFunc
}

func (vd *viiperDevice) Close() error {
	return vd.stream.Close()
}

func (vd *viiperDevice) WriteState() error {
	return vd.stream.WriteBinary(vd.state)
}

func (vd *viiperDevice) startRumbleReader(ctx context.Context) {
	rumbleCh, errCh := vd.stream.StartReading(ctx, 8, func(r *bufio.Reader) (encoding.BinaryUnmarshaler, error) {
		var b [2]byte
		if _, err := io.ReadFull(r, b[:]); err != nil {
			return nil, err
		}
		msg := new(xbox360.XRumbleState)
		if err := msg.UnmarshalBinary(b[:]); err != nil {
			return nil, err
		}
		return msg, nil
	})
	go func() {
		for {
			select {
			case v := <-rumbleCh:
				if v == nil {
					return
				}
				r := v.(*xbox360.XRumbleState)
				lf := uint16(r.LeftMotor) * 257
				rf := uint16(r.RightMotor) * 257
				_ = vd.pad.Rumble(lf, rf, 250)
			case e := <-errCh:
				if e != nil {
					slog.Debug("rumble end", "error", e)
				}
				return
			case <-ctx.Done():
				return
			}
		}
	}()
}

func (vd *viiperDevice) FromSDLGamepad(gp *sdl.Gamepad) error {
	if vd.state == nil {
		vd.state = &xbox360.InputState{}
	}
	if gp == nil {
		return nil
	}
	var b uint32
	if gp.Button(sdl.GAMEPAD_BUTTON_SOUTH) {
		b |= xbox360.ButtonA
	}
	if gp.Button(sdl.GAMEPAD_BUTTON_EAST) {
		b |= xbox360.ButtonB
	}
	if gp.Button(sdl.GAMEPAD_BUTTON_WEST) {
		b |= xbox360.ButtonX
	}
	if gp.Button(sdl.GAMEPAD_BUTTON_NORTH) {
		b |= xbox360.ButtonY
	}
	if gp.Button(sdl.GAMEPAD_BUTTON_START) {
		b |= xbox360.ButtonStart
	}
	if gp.Button(sdl.GAMEPAD_BUTTON_BACK) {
		b |= xbox360.ButtonBack
	}
	if gp.Button(sdl.GAMEPAD_BUTTON_LEFT_STICK) {
		b |= xbox360.ButtonLThumb
	}
	if gp.Button(sdl.GAMEPAD_BUTTON_RIGHT_STICK) {
		b |= xbox360.ButtonRThumb
	}
	if gp.Button(sdl.GAMEPAD_BUTTON_LEFT_SHOULDER) {
		b |= xbox360.ButtonLShoulder
	}
	if gp.Button(sdl.GAMEPAD_BUTTON_RIGHT_SHOULDER) {
		b |= xbox360.ButtonRShoulder
	}
	if gp.Button(sdl.GAMEPAD_BUTTON_GUIDE) {
		b |= xbox360.ButtonGuide
	}
	if gp.Button(sdl.GAMEPAD_BUTTON_DPAD_UP) {
		b |= xbox360.ButtonDPadUp
	}
	if gp.Button(sdl.GAMEPAD_BUTTON_DPAD_DOWN) {
		b |= xbox360.ButtonDPadDown
	}
	if gp.Button(sdl.GAMEPAD_BUTTON_DPAD_LEFT) {
		b |= xbox360.ButtonDPadLeft
	}
	if gp.Button(sdl.GAMEPAD_BUTTON_DPAD_RIGHT) {
		b |= xbox360.ButtonDPadRight
	}
	lt := gp.Axis(sdl.GAMEPAD_AXIS_LEFT_TRIGGER)
	rt := gp.Axis(sdl.GAMEPAD_AXIS_RIGHT_TRIGGER)
	if lt < 0 {
		lt = 0
	}
	if rt < 0 {
		rt = 0
	}

	vd.state.Buttons = b
	vd.state.LT = uint8((int(lt) * 255) / 32767)
	vd.state.RT = uint8((int(rt) * 255) / 32767)

	// Invert Y axes to match XInput convention
	// XInput: Negative values signify down or to the left. Positive values signify up or to the right.
	//         https://learn.microsoft.com/en-us/windows/win32/api/xinput/ns-xinput-xinput_gamepad
	// SDL: For thumbsticks, the state is a value ranging from -32768 (up/left) to 32767 (down/right).
	//      https://wiki.libsdl.org/SDL3/SDL_GetGamepadAxis

	vd.state.LX = int16(gp.Axis(sdl.GAMEPAD_AXIS_LEFTX))
	vd.state.LY = int16(gp.Axis(sdl.GAMEPAD_AXIS_LEFTY) * -1)
	vd.state.RX = int16(gp.Axis(sdl.GAMEPAD_AXIS_RIGHTX))
	vd.state.RY = int16(gp.Axis(sdl.GAMEPAD_AXIS_RIGHTY) * -1)
	return nil
}
