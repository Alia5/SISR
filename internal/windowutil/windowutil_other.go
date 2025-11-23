//go:build !windows

package windowutil

import "github.com/Zyko0/go-sdl3/sdl"

func MakeClickthrough(window *sdl.Window) error {
	panic("Not implemented")
}

func MakeClickable(window *sdl.Window) error {
	panic("Not implemented")
}
