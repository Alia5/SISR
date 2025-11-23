//go:build !windows

package main

type platformOpts struct {
	Console bool `kong:"-"`
}

func showConsole() error {
	panic("not implemented on non-Windows platforms")
}
