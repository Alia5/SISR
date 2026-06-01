package sisr

import (
	"context"
	"runtime"

	"github.com/Alia5/SISR/cmd"
	"github.com/Alia5/SISR/event"
	"github.com/Alia5/SISR/event/handler"
	"github.com/Alia5/SISR/event/handler/gamepadupdated"
	"github.com/Alia5/SISR/event/handler/sensorupdated"
	"github.com/Alia5/SISR/event/handler/touchpadupdated"
	"github.com/Alia5/SISR/sdl"
	"github.com/Alia5/SISR/sdl/extras"
	"github.com/Alia5/SISR/webview"
)

func registerEventHandlers(r event.Router, c *cmd.SISRContext, window *sdl.Window, wv webview.WebView) {
	if runtime.GOOS == "linux" {
		hittestfunc := handler.HandleFunc(
			func(_ context.Context, ev *sdl.WindowEvent) error {
				return extras.HandleCursorHitTestWindowEvent(window, ev)
			},
		)
		event.RegisterHandler(r, handler.Operation[*sdl.WindowEvent]{
			Event:   sdl.EventTypeWindowPixelSizeChanged,
			Handler: hittestfunc,
		})
		event.RegisterHandler(r, handler.Operation[*sdl.WindowEvent]{
			Event:   sdl.EventTypeWindowResized,
			Handler: hittestfunc,
		})
	}
	event.RegisterHandler(r, handler.Quit(c))
	event.RegisterHandler(r, handler.WindowResize(c, wv))
	event.RegisterHandler(r, handler.GamepadAdded(c))
	event.RegisterHandler(r, handler.GamepadRemoved(c))
	event.RegisterHandler(r, handler.ToggleUIKeyboardDown(c))
	event.RegisterHandler(r, handler.ToggleUIKeyboardUp())
	event.RegisterHandler(r, handler.ToggleUIGamepadButtonDown(c))
	event.RegisterHandler(r, handler.ToggleUIGamepadButtonUp())
	event.RegisterHandler(r, gamepadupdated.GamepadUpdated(c))
	event.RegisterHandler(r, sensorupdated.SensorUpdated(c))
	event.RegisterHandler(r, touchpadupdated.TouchpadDown(c))
	event.RegisterHandler(r, touchpadupdated.TouchpadMotion(c))
	event.RegisterHandler(r, touchpadupdated.TouchpadUp(c))

	if c.Config.KeyboardMouseEmulation {
		registerKBMForwardHandlers(r, c, window)
	}
}
