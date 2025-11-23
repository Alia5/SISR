//go:build windows

package windowutil

import (
	"log/slog"
	"syscall"
	"unsafe"

	"github.com/Zyko0/go-sdl3/sdl"
)

const (
	GWL_EXSTYLE = -20

	WS_EX_LAYERED     = 0x00080000
	WS_EX_TRANSPARENT = 0x00000020
	WS_EX_TOPMOST     = 0x00000008
	WS_EX_COMPOSITED  = 0x02000000

	HWND_TOPMOST = ^uintptr(0)

	SWP_NOMOVE     = 0x0002
	SWP_NOSIZE     = 0x0001
	SWP_NOACTIVATE = 0x0010
)

var user32 *syscall.LazyDLL
var sdl3 *syscall.LazyDLL
var setWindowLongProc *syscall.LazyProc
var setWindowPosProc *syscall.LazyProc
var getWindowPropsProc *syscall.LazyProc
var getPointerPropertyProc *syscall.LazyProc

func init() {
	user32 = syscall.NewLazyDLL("user32.dll")
	sdl3 = syscall.NewLazyDLL("SDL3.dll")
	setWindowLongProc = user32.NewProc("SetWindowLongW")
	setWindowPosProc = user32.NewProc("SetWindowPos")
	getWindowPropsProc = sdl3.NewProc("SDL_GetWindowProperties")
	getPointerPropertyProc = sdl3.NewProc("SDL_GetPointerProperty")
}

func MakeClickthrough(window *sdl.Window) error {
	hwnd, err := getWinowHWND(window)
	if err != nil {
		return err
	}

	exStyle := WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_COMPOSITED
	_, err = setWindowLongW(hwnd, GWL_EXSTYLE, uintptr(exStyle))
	if err != nil {
		return err
	}

	_, err = setWindowPos(
		hwnd,
		syscall.Handle(HWND_TOPMOST),
		0, 0, 0, 0,
		SWP_NOMOVE|SWP_NOSIZE|SWP_NOACTIVATE,
	)
	if err != nil {
		return err
	}

	return nil
}

func MakeClickable(window *sdl.Window) error {
	hwnd, err := getWinowHWND(window)
	if err != nil {
		return err
	}

	exStyle := WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_COMPOSITED
	_, err = setWindowLongW(hwnd, GWL_EXSTYLE, uintptr(exStyle))
	if err != nil {
		return err
	}

	_, err = setWindowPos(
		hwnd,
		syscall.Handle(HWND_TOPMOST),
		0, 0, 0, 0,
		SWP_NOMOVE|SWP_NOSIZE|SWP_NOACTIVATE,
	)
	if err != nil {
		return err
	}

	return nil
}

func setWindowLongW(hWnd syscall.Handle, nIndex int32, dwNewLong uintptr) (uintptr, error) {
	ret, _, err := setWindowLongProc.Call(
		uintptr(hWnd),
		uintptr(nIndex),
		dwNewLong,
	)
	if ret == 0 {
		return 0, err
	}
	return ret, nil
}

func setWindowPos(hWnd syscall.Handle, hWndInsertAfter syscall.Handle, X, Y, cx, cy int32, uFlags uint32) (bool, error) {
	bRet, _, err := setWindowPosProc.Call(
		uintptr(hWnd),
		uintptr(hWndInsertAfter),
		uintptr(X),
		uintptr(Y),
		uintptr(cx),
		uintptr(cy),
		uintptr(uFlags),
	)
	if err != syscall.Errno(0) {
		return bRet != 0, err
	}
	return bRet != 0, nil
}

func getWinowHWND(window *sdl.Window) (syscall.Handle, error) {
	propsID, _, _ := getWindowPropsProc.Call(uintptr(unsafe.Pointer(window)))
	if propsID == 0 {
		return 0, syscall.EINVAL
	}
	propName := append([]byte("SDL.window.win32.hwnd"), 0)
	hwndPtr, _, _ := getPointerPropertyProc.Call(propsID, uintptr(unsafe.Pointer(&propName[0])), 0)
	if hwndPtr == 0 {
		return 0, syscall.EINVAL
	}
	hwnd := syscall.Handle(hwndPtr)
	slog.Debug("hwnd", "value", hwnd)
	return hwnd, nil
}
