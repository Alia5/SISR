package tray

import "fyne.io/systray"

type Signal int

const (
	SignalQuit Signal = iota
	SignalExit
)

func Setup(enabled bool) (exitChan chan Signal) {
	if !enabled {
		return nil
	}
	trayReady := make(chan struct{})
	exitChan = make(chan Signal)
	go systray.Run(func() {
		trayReady <- struct{}{}
	}, func() {
		exitChan <- SignalExit
	})
	for range trayReady {
		break
	}
	setup()
	mQuit := systray.AddMenuItem("Quit", "Quit SISR")
	go func() {
		<-mQuit.ClickedCh
		exitChan <- SignalQuit
	}()

	return exitChan
}
