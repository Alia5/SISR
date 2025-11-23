//go:build windows

package main

import (
	"errors"
	"syscall"
)

type platformOpts struct {
	Console bool `help:"Show terminal Window (default: false)" default:"false" env:"SISR_CONSOLE"`
}

func showConsole() error {
	kernel32, err := syscall.LoadLibrary("kernel32.dll")
	if err != nil {
		return err
	}
	defer syscall.FreeLibrary(kernel32)

	proc, err := syscall.GetProcAddress(kernel32, "AllocConsole")
	if err != nil {
		return err
	}
	_, _, sErr := syscall.SyscallN(proc)
	if sErr == syscall.ERROR_ACCESS_DENIED {
		// Console already allocated
		return nil
	}
	if sErr != 0 {
		return errors.New("Failed to alloc console")
	}
	return nil
}
