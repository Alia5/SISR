//go:build windows

package main

import (
	"errors"
	"os"
	"syscall"
	"unsafe"
)

type platformOpts struct {
	Console bool `help:"Show terminal Window (default: false)" default:"false" env:"SISR_CONSOLE"`
}

const (
	STD_OUTPUT_HANDLE     = ^uintptr(10) // -11 as uintptr
	STD_ERROR_HANDLE      = ^uintptr(11) // -12 as uintptr
	DUPLICATE_SAME_ACCESS = 2
)

var (
	kernel32              = syscall.MustLoadDLL("kernel32.dll")
	procAllocConsole      = kernel32.MustFindProc("AllocConsole")
	procGetStdHandle      = kernel32.MustFindProc("GetStdHandle")
	procSetStdHandle      = kernel32.MustFindProc("SetStdHandle")
	procCreatePipe        = kernel32.MustFindProc("CreatePipe")
	procDuplicateHandle   = kernel32.MustFindProc("DuplicateHandle")
	procGetCurrentProcess = kernel32.MustFindProc("GetCurrentProcess")
)

func showConsole() error {
	// Allocate console (ignore if already exists)
	r1, _, _ := procAllocConsole.Call()
	if r1 == 0 {
		// Likely ERROR_ACCESS_DENIED - console already exists; continue anyway
	}

	// Create pipes for stdout and stderr
	var hReadPipeOut, hWritePipeOut, hReadPipeErr, hWritePipeErr syscall.Handle
	r1, _, _ = procCreatePipe.Call(
		uintptr(unsafe.Pointer(&hReadPipeOut)),
		uintptr(unsafe.Pointer(&hWritePipeOut)),
		0, 0,
	)
	if r1 == 0 {
		return errors.New("failed to create stdout pipe")
	}
	r1, _, _ = procCreatePipe.Call(
		uintptr(unsafe.Pointer(&hReadPipeErr)),
		uintptr(unsafe.Pointer(&hWritePipeErr)),
		0, 0,
	)
	if r1 == 0 {
		return errors.New("failed to create stderr pipe")
	}

	// Duplicate original stdout/stderr handles before redirect
	currentProc, _, _ := procGetCurrentProcess.Call()
	origStdout, _, _ := procGetStdHandle.Call(STD_OUTPUT_HANDLE)
	origStderr, _, _ := procGetStdHandle.Call(STD_ERROR_HANDLE)
	var dupStdout, dupStderr syscall.Handle
	r1, _, _ = procDuplicateHandle.Call(
		currentProc, origStdout, currentProc,
		uintptr(unsafe.Pointer(&dupStdout)), 0, 1, DUPLICATE_SAME_ACCESS,
	)
	if r1 == 0 {
		return errors.New("failed to duplicate stdout")
	}
	r1, _, _ = procDuplicateHandle.Call(
		currentProc, origStderr, currentProc,
		uintptr(unsafe.Pointer(&dupStderr)), 0, 1, DUPLICATE_SAME_ACCESS,
	)
	if r1 == 0 {
		return errors.New("failed to duplicate stderr")
	}

	// Redirect stdout/stderr to write ends of pipes
	procSetStdHandle.Call(STD_OUTPUT_HANDLE, uintptr(hWritePipeOut))
	procSetStdHandle.Call(STD_ERROR_HANDLE, uintptr(hWritePipeErr))

	// Rebind os.Stdout/Stderr to the duplicated original handles for visible output
	os.Stdout = os.NewFile(uintptr(dupStdout), "/dev/stdout")
	os.Stderr = os.NewFile(uintptr(dupStderr), "/dev/stderr")

	// Launch background copiers from pipe read ends → original duplicates
	go copyPipeToFile(hReadPipeOut, dupStdout)
	go copyPipeToFile(hReadPipeErr, dupStderr)

	return nil
}

func copyPipeToFile(hPipe, hFile syscall.Handle) {
	buf := make([]byte, 4096)
	for {
		var read uint32
		err := syscall.ReadFile(hPipe, buf, &read, nil)
		if err != nil || read == 0 {
			break
		}
		var written uint32
		syscall.WriteFile(hFile, buf[:read], &written, nil)
	}
}
