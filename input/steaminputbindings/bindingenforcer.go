package steaminputbindings

import (
	"log/slog"
	"os"
	"strconv"
	"sync"

	"github.com/Alia5/SISR/helper"
)

type Enforcer interface {
	GetForcedInputAppID() uint32
	ForceInputAppID(appID uint32) error
	ForceOwnAppID() error
}

type enforcer struct {
	forcedAppID uint32
	ownAppID    uint32
	noSteam     bool

	mtx sync.Mutex
}

func NewEnforcer(noSteam bool) Enforcer {
	var ownAppID uint32
	ownAppIDStr := os.Getenv("SteamAppId")
	if ownAppIDStr == "" || ownAppIDStr == "0" {
		ownAppIDStr = os.Getenv("SISR_MARKER_ID")
	}
	if ownAppIDStr != "" {
		appID64, err := strconv.ParseUint(ownAppIDStr, 10, 32)
		if err != nil {
			slog.Error("Failed to parse steamAppIDEnv", "SteamAppID", ownAppIDStr, "error", err)
		} else {
			ownAppID = uint32(appID64)
		}
	}
	return &enforcer{
		ownAppID: ownAppID,
		noSteam:  noSteam,
	}
}

func (e *enforcer) GetForcedInputAppID() uint32 {
	e.mtx.Lock()
	defer e.mtx.Unlock()
	return e.forcedAppID
}

func (e *enforcer) ForceInputAppID(appID uint32) error {
	e.mtx.Lock()
	defer e.mtx.Unlock()

	if e.noSteam {
		return nil
	}

	if appID == 0 {
		slog.Info("Unforcing SteamInput layout")
	} else {
		slog.Info("Forcing SteamInput layout for appID", "appID", appID)
	}

	err := helper.OpenURL("steam://forceinputappid/" + strconv.FormatUint(uint64(appID), 10))
	if err != nil {
		return err
	}

	e.forcedAppID = appID
	return nil
}

func (e *enforcer) ForceOwnAppID() error {
	e.mtx.Lock()
	defer e.mtx.Unlock()

	if e.noSteam {
		return nil
	}

	if e.ownAppID == 0 {
		return ErrNoSteamAppID
	}

	slog.Info("Forcing SteamInput layout for own appID", "appID", e.ownAppID)

	err := helper.OpenURL("steam://forceinputappid/" + strconv.FormatUint(uint64(e.ownAppID), 10))
	if err != nil {
		return err
	}

	e.forcedAppID = e.ownAppID
	return nil
}
