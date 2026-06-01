package config

type UpdateNotify string

const (
	UpdateNotifyNone       UpdateNotify = "none"
	UpdateNotifyStable     UpdateNotify = "stable"
	UpdateNotifyPrerelease UpdateNotify = "prerelease"
	UpdateNotifyAlways     UpdateNotify = "always"
)

type Log struct {
	Level string `aliases:"l" default:"info" help:"Log level: trace, debug, info, warn, error" env:"SISR_LOG_LEVEL"`
	File  string `aliases:"lf" help:"Log file path" env:"SISR_LOG_FILE"`
}

type Global struct {
	ConfigPath   string `help:"Path to configuration file (json|yaml|toml)" name:"config" env:"SISR_CONFIG"`
	Log          `embed:"" prefix:"log."`
	PlatformOpts `embed:""`
	Marker       bool `help:"dummy, used for Steam Input layout" hidden:""`
}

type API struct {
	ListenAddress   string `default:"localhost:0" help:"API listen address; by default will pick random free port and listen on localhost only" env:"SISR_API_LISTEN_ADDRESS"`
	CORSOrigins     string `help:"CORS allowed origins" default:"https://steamloopback.host,http://steamloopback.host" env:"SISR_API_CORS_ORIGINS"`
	FrontendAddress string `help:"Frontend address (optional)" hidden:""`
}

type Steam struct {
	InstallDir         string `help:"Steam installation directory (optional, will attempt to auto-detect if not set)" env:"SISR_STEAM_INSTALL_DIR"`
	UserID             uint32 `help:"Active Steam user ID (optional, will attempt to auto-detect if not set)" env:"SISR_STEAM_USER_ID"`
	CEFRemoteDebugPort uint16 `help:"CEF remote debugging port (optional, will attempt to auto-detect if not set)" env:"SISR_STEAM_CEF_REMOTE_DEBUG_PORT"`
}

type RunMisc struct {
	NoSteam       bool `default:"false" help:"Run in no-Steam mode" env:"SISR_NO_STEAM"`
	InitialLaunch bool `default:"false" hidden:""`
}

type AutoUpdate struct {
	UpdateNotify UpdateNotify `default:"stable" enum:"none,stable,prerelease,always" help:"Update notification level" env:"SISR_UPDATE_NOTIFY"`
}

type Window struct {
	MaxFPS     uint32 `default:"60" help:"Maximim FPS for SteamOverlay/UI (Does not affect inputs)" env:"SISR_MAX_FPS"`
	Fullscreen bool   `default:"true" aliases:"f" help:"Fullscreen overlay"  env:"SISR_FULLSCREEN"`
	Show       bool   `default:"false" aliases:"w" help:"Shows window on startup; when used w/ fullscreen=true: will enable Steam Overlay; when used with fullscreen=false: will show UI; when false SISR will only show up in system tray"   env:"SISR_SHOW_WINDOW"`
}

type ControllerEmulation struct {
	DefaultControllerType string `default:"xbox360" aliases:"ct" enum:"xbox360,dualshock4,dualsense,dualsenseedge" help:"Default controller type for emulation" env:"SISR_DEFAULT_CONTROLLER_TYPE"`

	GyroPassthrough     bool `default:"true" help:"Enable gyro passthrough for supported controllers" env:"SISR_GYRO_PASSTHROUGH"`

	AllowSteamDesktopLayout bool `default:"false" help:"Allow/Use Steam's desktop configuration for emulated controllers" env:"SISR_ALLOW_STEAM_DESKTOP_LAYOUT"`
	ImmediateSensorUpdates  bool `default:"true" help:"Immediately send sensor updates to VIIPER instead of waiting for the next input report; may reduce latency at the cost of increased CPU usage" env:"SISR_IMMEDIATE_SENSOR_UPDATES"`
}

type Viiper struct {
	Address  string `default:"localhost:3242" aliases:"va" help:"VIIPER server address" env:"SISR_VIIPER_ADDRESS"`
	Password string `aliases:"vp" help:"VIIPER server password" env:"SISR_VIIPER_PASSWORD"`
}

type KbMEmuation struct {
	KeyboardMouseEmulation bool `default:"false" aliases:"kbm" help:"Forward Keyboard/Mouse when running over a network" env:"SISR_KBM_EMULATION"`
}
