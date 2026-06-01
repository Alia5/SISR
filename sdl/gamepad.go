package sdl

/*
#cgo CFLAGS: -I${SRCDIR}/../deps/SDL/include

#include <stdlib.h>

#include <SDL3/SDL_gamepad.h>
#include <SDL3/SDL_sensor.h>

static inline int gamepad_binding_input_button(const SDL_GamepadBinding *b)
{
	return b->input.button;
}

static inline int gamepad_binding_input_axis(const SDL_GamepadBinding *b)
{
	return b->input.axis.axis;
}

static inline int gamepad_binding_input_axis_min(const SDL_GamepadBinding *b)
{
	return b->input.axis.axis_min;
}

static inline int gamepad_binding_input_axis_max(const SDL_GamepadBinding *b)
{
	return b->input.axis.axis_max;
}

static inline int gamepad_binding_input_hat(const SDL_GamepadBinding *b)
{
	return b->input.hat.hat;
}

static inline int gamepad_binding_input_hat_mask(const SDL_GamepadBinding *b)
{
	return b->input.hat.hat_mask;
}

static inline int gamepad_binding_output_button(const SDL_GamepadBinding *b)
{
	return (int)b->output.button;
}

static inline int gamepad_binding_output_axis(const SDL_GamepadBinding *b)
{
	return (int)b->output.axis.axis;
}

static inline int gamepad_binding_output_axis_min(const SDL_GamepadBinding *b)
{
	return b->output.axis.axis_min;
}

static inline int gamepad_binding_output_axis_max(const SDL_GamepadBinding *b)
{
	return b->output.axis.axis_max;
}
*/
import "C"

import "unsafe"

type GamepadID int32

// GamepadType standard gamepad types.
type GamepadType int32

// GamepadAxis the list of axes available on a gamepad.
type GamepadAxis int32

// GamepadButton the list of buttons available on a gamepad.
type GamepadButton int32

// GamepadButtonLabel the set of gamepad button labels.
type GamepadButtonLabel int32

// GamepadBindingType describes the type of a gamepad control binding.
type GamepadBindingType int32

type SensorType int32

const (
	GamepadTypeUnknown                   GamepadType = C.SDL_GAMEPAD_TYPE_UNKNOWN
	GamepadTypeStandard                  GamepadType = C.SDL_GAMEPAD_TYPE_STANDARD
	GamepadTypeXbox360                   GamepadType = C.SDL_GAMEPAD_TYPE_XBOX360
	GamepadTypeXboxOne                   GamepadType = C.SDL_GAMEPAD_TYPE_XBOXONE
	GamepadTypePS3                       GamepadType = C.SDL_GAMEPAD_TYPE_PS3
	GamepadTypePS4                       GamepadType = C.SDL_GAMEPAD_TYPE_PS4
	GamepadTypePS5                       GamepadType = C.SDL_GAMEPAD_TYPE_PS5
	GamepadTypeNintendoSwitchPro         GamepadType = C.SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_PRO
	GamepadTypeNintendoSwitchJoyconLeft  GamepadType = C.SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_JOYCON_LEFT
	GamepadTypeNintendoSwitchJoyconRight GamepadType = C.SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_JOYCON_RIGHT
	GamepadTypeNintendoSwitchJoyconPair  GamepadType = C.SDL_GAMEPAD_TYPE_NINTENDO_SWITCH_JOYCON_PAIR
	GamepadTypeGameCube                  GamepadType = C.SDL_GAMEPAD_TYPE_GAMECUBE
)

const (
	GamepadAxisInvalid      GamepadAxis = C.SDL_GAMEPAD_AXIS_INVALID
	GamepadAxisLeftX        GamepadAxis = C.SDL_GAMEPAD_AXIS_LEFTX
	GamepadAxisLeftY        GamepadAxis = C.SDL_GAMEPAD_AXIS_LEFTY
	GamepadAxisRightX       GamepadAxis = C.SDL_GAMEPAD_AXIS_RIGHTX
	GamepadAxisRightY       GamepadAxis = C.SDL_GAMEPAD_AXIS_RIGHTY
	GamepadAxisLeftTrigger  GamepadAxis = C.SDL_GAMEPAD_AXIS_LEFT_TRIGGER
	GamepadAxisRightTrigger GamepadAxis = C.SDL_GAMEPAD_AXIS_RIGHT_TRIGGER
)

const (
	GamepadButtonInvalid       GamepadButton = C.SDL_GAMEPAD_BUTTON_INVALID
	GamepadButtonSouth         GamepadButton = C.SDL_GAMEPAD_BUTTON_SOUTH
	GamepadButtonEast          GamepadButton = C.SDL_GAMEPAD_BUTTON_EAST
	GamepadButtonWest          GamepadButton = C.SDL_GAMEPAD_BUTTON_WEST
	GamepadButtonNorth         GamepadButton = C.SDL_GAMEPAD_BUTTON_NORTH
	GamepadButtonBack          GamepadButton = C.SDL_GAMEPAD_BUTTON_BACK
	GamepadButtonGuide         GamepadButton = C.SDL_GAMEPAD_BUTTON_GUIDE
	GamepadButtonStart         GamepadButton = C.SDL_GAMEPAD_BUTTON_START
	GamepadButtonLeftStick     GamepadButton = C.SDL_GAMEPAD_BUTTON_LEFT_STICK
	GamepadButtonRightStick    GamepadButton = C.SDL_GAMEPAD_BUTTON_RIGHT_STICK
	GamepadButtonLeftShoulder  GamepadButton = C.SDL_GAMEPAD_BUTTON_LEFT_SHOULDER
	GamepadButtonRightShoulder GamepadButton = C.SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER
	GamepadButtonDpadUp        GamepadButton = C.SDL_GAMEPAD_BUTTON_DPAD_UP
	GamepadButtonDpadDown      GamepadButton = C.SDL_GAMEPAD_BUTTON_DPAD_DOWN
	GamepadButtonDpadLeft      GamepadButton = C.SDL_GAMEPAD_BUTTON_DPAD_LEFT
	GamepadButtonDpadRight     GamepadButton = C.SDL_GAMEPAD_BUTTON_DPAD_RIGHT
	GamepadButtonMisc1         GamepadButton = C.SDL_GAMEPAD_BUTTON_MISC1
	GamepadButtonRightPaddle1  GamepadButton = C.SDL_GAMEPAD_BUTTON_RIGHT_PADDLE1
	GamepadButtonLeftPaddle1   GamepadButton = C.SDL_GAMEPAD_BUTTON_LEFT_PADDLE1
	GamepadButtonRightPaddle2  GamepadButton = C.SDL_GAMEPAD_BUTTON_RIGHT_PADDLE2
	GamepadButtonLeftPaddle2   GamepadButton = C.SDL_GAMEPAD_BUTTON_LEFT_PADDLE2
	GamepadButtonTouchpad      GamepadButton = C.SDL_GAMEPAD_BUTTON_TOUCHPAD
	GamepadButtonMisc2         GamepadButton = C.SDL_GAMEPAD_BUTTON_MISC2
	GamepadButtonMisc3         GamepadButton = C.SDL_GAMEPAD_BUTTON_MISC3
	GamepadButtonMisc4         GamepadButton = C.SDL_GAMEPAD_BUTTON_MISC4
	GamepadButtonMisc5         GamepadButton = C.SDL_GAMEPAD_BUTTON_MISC5
	GamepadButtonMisc6         GamepadButton = C.SDL_GAMEPAD_BUTTON_MISC6
)

const (
	GamepadButtonLabelUnknown  GamepadButtonLabel = C.SDL_GAMEPAD_BUTTON_LABEL_UNKNOWN
	GamepadButtonLabelA        GamepadButtonLabel = C.SDL_GAMEPAD_BUTTON_LABEL_A
	GamepadButtonLabelB        GamepadButtonLabel = C.SDL_GAMEPAD_BUTTON_LABEL_B
	GamepadButtonLabelX        GamepadButtonLabel = C.SDL_GAMEPAD_BUTTON_LABEL_X
	GamepadButtonLabelY        GamepadButtonLabel = C.SDL_GAMEPAD_BUTTON_LABEL_Y
	GamepadButtonLabelCross    GamepadButtonLabel = C.SDL_GAMEPAD_BUTTON_LABEL_CROSS
	GamepadButtonLabelCircle   GamepadButtonLabel = C.SDL_GAMEPAD_BUTTON_LABEL_CIRCLE
	GamepadButtonLabelSquare   GamepadButtonLabel = C.SDL_GAMEPAD_BUTTON_LABEL_SQUARE
	GamepadButtonLabelTriangle GamepadButtonLabel = C.SDL_GAMEPAD_BUTTON_LABEL_TRIANGLE
)

const (
	GamepadBindingTypeNone   GamepadBindingType = C.SDL_GAMEPAD_BINDTYPE_NONE
	GamepadBindingTypeButton GamepadBindingType = C.SDL_GAMEPAD_BINDTYPE_BUTTON
	GamepadBindingTypeAxis   GamepadBindingType = C.SDL_GAMEPAD_BINDTYPE_AXIS
	GamepadBindingTypeHat    GamepadBindingType = C.SDL_GAMEPAD_BINDTYPE_HAT
)

const (
	SensorTypeAccelerometer SensorType = C.SDL_SENSOR_ACCEL
	SensorTypeGyroscope     SensorType = C.SDL_SENSOR_GYRO
)

// GamepadBinding describes one joystick-layer binding for a gamepad.
type GamepadBinding struct {
	InputType    GamepadBindingType
	InputButton  int
	InputAxis    int
	InputAxisMin int
	InputAxisMax int
	InputHat     int
	InputHatMask int

	OutputType    GamepadBindingType
	OutputButton  GamepadButton
	OutputAxis    GamepadAxis
	OutputAxisMin int
	OutputAxisMax int
}

// Gamepad the structure used to identify an SDL gamepad.
type Gamepad struct {
	cGamepad *C.SDL_Gamepad
}

func InitGamepadSubSystem() error {
	return InitSubSystem(InitFlagGamepad)
}

func QuitGamepadSubSystem() {
	QuitSubSystem(InitFlagGamepad)
}

// HasGamepad returns whether a gamepad is currently connected.
func HasGamepad() bool {
	return bool(C.SDL_HasGamepad())
}

// GetGamepads returns a list of currently connected gamepads.
func GetGamepads() ([]GamepadID, error) {
	var count C.int
	cIDs := C.SDL_GetGamepads(&count)
	if cIDs == nil {
		if count == 0 {
			return []GamepadID{}, nil
		}
		return nil, GetError()
	}
	defer C.SDL_free(unsafe.Pointer(cIDs))

	cSlice := unsafe.Slice(cIDs, int(count))
	ids := make([]GamepadID, 0, int(count))
	for _, id := range cSlice {
		ids = append(ids, GamepadID(id))
	}
	return ids, nil
}

// IsGamepad checks if the given joystick is supported by the gamepad interface.
func IsGamepad(id GamepadID) bool {
	return bool(C.SDL_IsGamepad(C.SDL_JoystickID(id)))
}

// GetGamepadNameForID gets the implementation dependent name of a gamepad.
//
// This can be called before any gamepads are opened.
func GetGamepadNameForID(id GamepadID) string {
	cName := C.SDL_GetGamepadNameForID(C.SDL_JoystickID(id))
	if cName == nil {
		return ""
	}
	return C.GoString(cName)
}

// OpenGamepad opens a gamepad for use.
func OpenGamepad(id GamepadID) (*Gamepad, error) {
	cg := C.SDL_OpenGamepad(C.SDL_JoystickID(id))
	if cg == nil {
		return nil, GetError()
	}
	return &Gamepad{cGamepad: cg}, nil
}

// GetGamepadFromID gets the SDL_Gamepad associated with a joystick instance ID, if it has been opened.
func GetGamepadFromID(id GamepadID) (*Gamepad, bool) {
	cg := C.SDL_GetGamepadFromID(C.SDL_JoystickID(id))
	if cg == nil {
		return nil, false
	}
	return &Gamepad{cGamepad: cg}, true
}

// SetGamepadEventsEnabled sets the state of gamepad event processing.
func SetGamepadEventsEnabled(enabled bool) {
	C.SDL_SetGamepadEventsEnabled(C.bool(enabled))
}

// GamepadEventsEnabled queries the state of gamepad event processing.
func GamepadEventsEnabled() bool {
	return bool(C.SDL_GamepadEventsEnabled())
}

// UpdateGamepads manually pumps gamepad updates if not using the loop.
func UpdateGamepads() {
	C.SDL_UpdateGamepads()
}

// Close closes a gamepad previously opened with SDL_OpenGamepad().
func (g *Gamepad) Close() {
	if g == nil || g.cGamepad == nil {
		return
	}
	C.SDL_CloseGamepad(g.cGamepad)
	g.cGamepad = nil
}

// Connected checks if a gamepad has been opened and is currently connected.
func (g *Gamepad) Connected() bool {
	if g == nil || g.cGamepad == nil {
		return false
	}
	return bool(C.SDL_GamepadConnected(g.cGamepad))
}

// ID gets the instance ID of an opened gamepad.
func (g *Gamepad) ID() GamepadID {
	if g == nil || g.cGamepad == nil {
		return 0
	}
	return GamepadID(C.SDL_GetGamepadID(g.cGamepad))
}

// Name gets the implementation-dependent name for an opened gamepad.
func (g *Gamepad) Name() string {
	if g == nil || g.cGamepad == nil {
		return ""
	}
	cName := C.SDL_GetGamepadName(g.cGamepad)
	if cName == nil {
		return ""
	}
	return C.GoString(cName)
}

// Type gets the type of an opened gamepad.
func (g *Gamepad) Type() GamepadType {
	if g == nil || g.cGamepad == nil {
		return GamepadTypeUnknown
	}
	return GamepadType(C.SDL_GetGamepadType(g.cGamepad))
}

// HasAxis queries whether a gamepad has a given axis.
func (g *Gamepad) HasAxis(axis GamepadAxis) bool {
	if g == nil || g.cGamepad == nil {
		return false
	}
	return bool(C.SDL_GamepadHasAxis(g.cGamepad, C.SDL_GamepadAxis(axis)))
}

// GetAxis gets the current state of an axis control on a gamepad.
func (g *Gamepad) GetAxis(axis GamepadAxis) int16 {
	if g == nil || g.cGamepad == nil {
		return 0
	}
	return int16(C.SDL_GetGamepadAxis(g.cGamepad, C.SDL_GamepadAxis(axis)))
}

// HasButton queries whether a gamepad has a given button.
func (g *Gamepad) HasButton(button GamepadButton) bool {
	if g == nil || g.cGamepad == nil {
		return false
	}
	return bool(C.SDL_GamepadHasButton(g.cGamepad, C.SDL_GamepadButton(button)))
}

// GetButton gets the current state of a button on a gamepad.
func (g *Gamepad) GetButton(button GamepadButton) bool {
	if g == nil || g.cGamepad == nil {
		return false
	}
	return bool(C.SDL_GetGamepadButton(g.cGamepad, C.SDL_GamepadButton(button)))
}

// GetButtonLabel gets the label of a button on a gamepad.
func (g *Gamepad) GetButtonLabel(button GamepadButton) GamepadButtonLabel {
	if g == nil || g.cGamepad == nil {
		return GamepadButtonLabelUnknown
	}
	return GamepadButtonLabel(C.SDL_GetGamepadButtonLabel(g.cGamepad, C.SDL_GamepadButton(button)))
}

// SetPlayerIndex sets the player index of an opened gamepad.
func (g *Gamepad) SetPlayerIndex(index int) error {
	if g == nil || g.cGamepad == nil {
		return &SDLError{eStr: "invalid gamepad handle"}
	}
	if !C.SDL_SetGamepadPlayerIndex(g.cGamepad, C.int(index)) {
		return GetError()
	}
	return nil
}

// GetPlayerIndex gets the player index of an opened gamepad.
func (g *Gamepad) GetPlayerIndex() int {
	if g == nil || g.cGamepad == nil {
		return -1
	}
	return int(C.SDL_GetGamepadPlayerIndex(g.cGamepad))
}

// GetSteamHandle gets the Steam Input handle for an opened gamepad, if available.
//
// Returns 0 when unavailable.
func (g *Gamepad) GetSteamHandle() uint64 {
	if g == nil || g.cGamepad == nil {
		return 0
	}
	return uint64(C.SDL_GetGamepadSteamHandle(g.cGamepad))
}

// AddGamepadMapping adds or updates a gamepad mapping string.
func AddGamepadMapping(mapping string) (int, error) {
	cMapping := C.CString(mapping)
	defer C.free(unsafe.Pointer(cMapping))

	res := int(C.SDL_AddGamepadMapping(cMapping))
	if res < 0 {
		return res, GetError()
	}
	return res, nil
}

// AddGamepadMappingsFromIO loads gamepad mappings from an SDL_IOStream.
func AddGamepadMappingsFromIO(src unsafe.Pointer, closeIO bool) (int, error) {
	res := int(C.SDL_AddGamepadMappingsFromIO((*C.SDL_IOStream)(src), C.bool(closeIO)))
	if res < 0 {
		return res, GetError()
	}
	return res, nil
}

// AddGamepadMappingsFromFile loads gamepad mappings from a file.
func AddGamepadMappingsFromFile(file string) (int, error) {
	cFile := C.CString(file)
	defer C.free(unsafe.Pointer(cFile))

	res := int(C.SDL_AddGamepadMappingsFromFile(cFile))
	if res < 0 {
		return res, GetError()
	}
	return res, nil
}

// ReloadGamepadMappings reinitializes the gamepad mapping database.
func ReloadGamepadMappings() error {
	if !C.SDL_ReloadGamepadMappings() {
		return GetError()
	}
	return nil
}

// GetGamepadMappings returns all current gamepad mapping strings.
func GetGamepadMappings() ([]string, error) {
	var count C.int
	cMappings := C.SDL_GetGamepadMappings(&count)
	if cMappings == nil {
		if count == 0 {
			return []string{}, nil
		}
		return nil, GetError()
	}
	defer C.SDL_free(unsafe.Pointer(cMappings))

	cSlice := unsafe.Slice(cMappings, int(count))
	mappings := make([]string, 0, int(count))
	for _, m := range cSlice {
		if m == nil {
			mappings = append(mappings, "")
			continue
		}
		mappings = append(mappings, C.GoString(m))
	}

	return mappings, nil
}

// GetGamepadMappingForGUID gets the mapping string for a gamepad GUID.
func GetGamepadMappingForGUID(guid GUID) string {
	cg := *(*C.SDL_GUID)(unsafe.Pointer(&guid))
	cMapping := C.SDL_GetGamepadMappingForGUID(cg)
	if cMapping == nil {
		return ""
	}
	defer C.SDL_free(unsafe.Pointer(cMapping))
	return C.GoString(cMapping)
}

// GetGamepadPathForID gets the implementation dependent path of a gamepad.
func GetGamepadPathForID(id GamepadID) string {
	cPath := C.SDL_GetGamepadPathForID(C.SDL_JoystickID(id))
	if cPath == nil {
		return ""
	}
	return C.GoString(cPath)
}

// GetGamepadPlayerIndexForID gets the player index of a gamepad.
func GetGamepadPlayerIndexForID(id GamepadID) int {
	return int(C.SDL_GetGamepadPlayerIndexForID(C.SDL_JoystickID(id)))
}

// GetGamepadGUIDForID gets the implementation-dependent GUID of a gamepad.
func GetGamepadGUIDForID(id GamepadID) GUID {
	cg := C.SDL_GetGamepadGUIDForID(C.SDL_JoystickID(id))
	return *(*GUID)(unsafe.Pointer(&cg))
}

// GetGamepadVendorForID gets the USB vendor ID of a gamepad, if available.
func GetGamepadVendorForID(id GamepadID) uint16 {
	return uint16(C.SDL_GetGamepadVendorForID(C.SDL_JoystickID(id)))
}

// GetGamepadProductForID gets the USB product ID of a gamepad, if available.
func GetGamepadProductForID(id GamepadID) uint16 {
	return uint16(C.SDL_GetGamepadProductForID(C.SDL_JoystickID(id)))
}

// GetGamepadProductVersionForID gets the product version of a gamepad, if available.
func GetGamepadProductVersionForID(id GamepadID) uint16 {
	return uint16(C.SDL_GetGamepadProductVersionForID(C.SDL_JoystickID(id)))
}

// GetGamepadTypeForID gets the type of a gamepad.
func GetGamepadTypeForID(id GamepadID) GamepadType {
	return GamepadType(C.SDL_GetGamepadTypeForID(C.SDL_JoystickID(id)))
}

// GetRealGamepadTypeForID gets the type of a gamepad, ignoring mapping overrides.
func GetRealGamepadTypeForID(id GamepadID) GamepadType {
	return GamepadType(C.SDL_GetRealGamepadTypeForID(C.SDL_JoystickID(id)))
}

// GetGamepadMappingForID gets the mapping string for a gamepad ID.
func GetGamepadMappingForID(id GamepadID) string {
	cMapping := C.SDL_GetGamepadMappingForID(C.SDL_JoystickID(id))
	if cMapping == nil {
		return ""
	}
	defer C.SDL_free(unsafe.Pointer(cMapping))
	return C.GoString(cMapping)
}

// GetGamepadFromPlayerIndex gets the SDL_Gamepad associated with a player index.
func GetGamepadFromPlayerIndex(playerIndex int) (*Gamepad, bool) {
	cg := C.SDL_GetGamepadFromPlayerIndex(C.int(playerIndex))
	if cg == nil {
		return nil, false
	}
	return &Gamepad{cGamepad: cg}, true
}

// SetGamepadMapping sets the current mapping of a joystick or gamepad.
//
// Pass an empty mapping string to clear the mapping.
func SetGamepadMapping(id GamepadID, mapping string) error {
	var cMapping *C.char
	if mapping != "" {
		cMapping = C.CString(mapping)
		defer C.free(unsafe.Pointer(cMapping))
	}

	if !C.SDL_SetGamepadMapping(C.SDL_JoystickID(id), cMapping) {
		return GetError()
	}
	return nil
}

// GetGamepadTypeFromString converts a string to a GamepadType.
func GetGamepadTypeFromString(s string) GamepadType {
	cStr := C.CString(s)
	defer C.free(unsafe.Pointer(cStr))
	return GamepadType(C.SDL_GetGamepadTypeFromString(cStr))
}

// GetGamepadStringForType converts a GamepadType to a string.
func GetGamepadStringForType(t GamepadType) string {
	cStr := C.SDL_GetGamepadStringForType(C.SDL_GamepadType(t))
	if cStr == nil {
		return ""
	}
	return C.GoString(cStr)
}

// Name gets the string name of a gamepad type.
func (t GamepadType) Name() string {
	return GetGamepadStringForType(t)
}

// GetGamepadAxisFromString converts a string to a GamepadAxis.
func GetGamepadAxisFromString(s string) GamepadAxis {
	cStr := C.CString(s)
	defer C.free(unsafe.Pointer(cStr))
	return GamepadAxis(C.SDL_GetGamepadAxisFromString(cStr))
}

// GetGamepadStringForAxis converts a GamepadAxis to a string.
func GetGamepadStringForAxis(axis GamepadAxis) string {
	cStr := C.SDL_GetGamepadStringForAxis(C.SDL_GamepadAxis(axis))
	if cStr == nil {
		return ""
	}
	return C.GoString(cStr)
}

// GetGamepadButtonFromString converts a string to a GamepadButton.
func GetGamepadButtonFromString(s string) GamepadButton {
	cStr := C.CString(s)
	defer C.free(unsafe.Pointer(cStr))
	return GamepadButton(C.SDL_GetGamepadButtonFromString(cStr))
}

// GetGamepadStringForButton converts a GamepadButton to a string.
func GetGamepadStringForButton(button GamepadButton) string {
	cStr := C.SDL_GetGamepadStringForButton(C.SDL_GamepadButton(button))
	if cStr == nil {
		return ""
	}
	return C.GoString(cStr)
}

// GetGamepadButtonLabelForType gets the button label for a button on a gamepad type.
func GetGamepadButtonLabelForType(t GamepadType, button GamepadButton) GamepadButtonLabel {
	return GamepadButtonLabel(C.SDL_GetGamepadButtonLabelForType(C.SDL_GamepadType(t), C.SDL_GamepadButton(button)))
}

// Path gets the implementation-dependent path for an opened gamepad.
func (g *Gamepad) Path() string {
	if g == nil || g.cGamepad == nil {
		return ""
	}
	cPath := C.SDL_GetGamepadPath(g.cGamepad)
	if cPath == nil {
		return ""
	}
	return C.GoString(cPath)
}

// RealType gets the type of an opened gamepad, ignoring mapping override.
func (g *Gamepad) RealType() GamepadType {
	if g == nil || g.cGamepad == nil {
		return GamepadTypeUnknown
	}
	return GamepadType(C.SDL_GetRealGamepadType(g.cGamepad))
}

// Vendor gets the USB vendor ID of an opened gamepad, if available.
func (g *Gamepad) Vendor() uint16 {
	if g == nil || g.cGamepad == nil {
		return 0
	}
	return uint16(C.SDL_GetGamepadVendor(g.cGamepad))
}

// Product gets the USB product ID of an opened gamepad, if available.
func (g *Gamepad) Product() uint16 {
	if g == nil || g.cGamepad == nil {
		return 0
	}
	return uint16(C.SDL_GetGamepadProduct(g.cGamepad))
}

// ProductVersion gets the product version of an opened gamepad, if available.
func (g *Gamepad) ProductVersion() uint16 {
	if g == nil || g.cGamepad == nil {
		return 0
	}
	return uint16(C.SDL_GetGamepadProductVersion(g.cGamepad))
}

// FirmwareVersion gets the firmware version of an opened gamepad, if available.
func (g *Gamepad) FirmwareVersion() uint16 {
	if g == nil || g.cGamepad == nil {
		return 0
	}
	return uint16(C.SDL_GetGamepadFirmwareVersion(g.cGamepad))
}

// Serial gets the serial number of an opened gamepad, if available.
func (g *Gamepad) Serial() string {
	if g == nil || g.cGamepad == nil {
		return ""
	}
	cSerial := C.SDL_GetGamepadSerial(g.cGamepad)
	if cSerial == nil {
		return ""
	}
	return C.GoString(cSerial)
}

// ConnectionState gets the connection state of an opened gamepad.
func (g *Gamepad) ConnectionState() JoystickConnectionState {
	if g == nil || g.cGamepad == nil {
		return JoystickConnectionInvalid
	}
	return JoystickConnectionState(C.SDL_GetGamepadConnectionState(g.cGamepad))
}

// PowerInfo gets the battery state of an opened gamepad.
func (g *Gamepad) PowerInfo() (int, int) {
	if g == nil || g.cGamepad == nil {
		return -1, -1
	}
	var percent C.int
	state := C.SDL_GetGamepadPowerInfo(g.cGamepad, &percent)
	return int(state), int(percent)
}

// Joystick gets the underlying joystick from an opened gamepad.
func (g *Gamepad) Joystick() *Joystick {
	if g == nil || g.cGamepad == nil {
		return nil
	}
	cj := C.SDL_GetGamepadJoystick(g.cGamepad)
	if cj == nil {
		return nil
	}
	return &Joystick{cJoystick: cj}
}

// Mapping gets the current mapping of an opened gamepad.
func (g *Gamepad) Mapping() string {
	if g == nil || g.cGamepad == nil {
		return ""
	}
	cMapping := C.SDL_GetGamepadMapping(g.cGamepad)
	if cMapping == nil {
		return ""
	}
	defer C.SDL_free(unsafe.Pointer(cMapping))
	return C.GoString(cMapping)
}

// GetProperties gets the properties associated with an opened gamepad.
func (g *Gamepad) GetProperties() uintptr {
	if g == nil || g.cGamepad == nil {
		return 0
	}
	return uintptr(C.SDL_GetGamepadProperties(g.cGamepad))
}

// NumTouchpads gets the number of touchpads on a gamepad.
func (g *Gamepad) NumTouchpads() int {
	if g == nil || g.cGamepad == nil {
		return -1
	}
	return int(C.SDL_GetNumGamepadTouchpads(g.cGamepad))
}

// NumTouchpadFingers gets the number of simultaneous fingers supported on a gamepad touchpad.
func (g *Gamepad) NumTouchpadFingers(touchpad int) int {
	if g == nil || g.cGamepad == nil {
		return -1
	}
	return int(C.SDL_GetNumGamepadTouchpadFingers(g.cGamepad, C.int(touchpad)))
}

// GetTouchpadFinger gets the current state of a finger on a gamepad touchpad.
func (g *Gamepad) GetTouchpadFinger(touchpad, finger int) (bool, bool, float32, float32, float32) {
	if g == nil || g.cGamepad == nil {
		return false, false, 0, 0, 0
	}

	var down C.bool
	var x C.float
	var y C.float
	var pressure C.float
	ok := C.SDL_GetGamepadTouchpadFinger(g.cGamepad, C.int(touchpad), C.int(finger), &down, &x, &y, &pressure)
	return bool(ok), bool(down), float32(x), float32(y), float32(pressure)
}

// HasSensor queries whether a gamepad has a given sensor type.
func (g *Gamepad) HasSensor(sensorType SensorType) bool {
	if g == nil || g.cGamepad == nil {
		return false
	}
	return bool(C.SDL_GamepadHasSensor(g.cGamepad, C.SDL_SensorType(sensorType)))
}

// SetSensorEnabled sets whether sensor data reporting is enabled for a gamepad sensor.
func (g *Gamepad) SetSensorEnabled(sensorType SensorType, enabled bool) bool {
	if g == nil || g.cGamepad == nil {
		return false
	}
	return bool(C.SDL_SetGamepadSensorEnabled(g.cGamepad, C.SDL_SensorType(sensorType), C.bool(enabled)))
}

// SensorEnabled queries whether sensor data reporting is enabled for a gamepad sensor.
func (g *Gamepad) SensorEnabled(sensorType SensorType) bool {
	if g == nil || g.cGamepad == nil {
		return false
	}
	return bool(C.SDL_GamepadSensorEnabled(g.cGamepad, C.SDL_SensorType(sensorType)))
}

// SensorDataRate gets the data rate of a gamepad sensor.
func (g *Gamepad) SensorDataRate(sensorType SensorType) float32 {
	if g == nil || g.cGamepad == nil {
		return 0
	}
	return float32(C.SDL_GetGamepadSensorDataRate(g.cGamepad, C.SDL_SensorType(sensorType)))
}

// GetSensorData gets the current state of a gamepad sensor.
func (g *Gamepad) GetSensorData(sensorType SensorType, values []float32) bool {
	if g == nil || g.cGamepad == nil {
		return false
	}
	if len(values) == 0 {
		return bool(C.SDL_GetGamepadSensorData(g.cGamepad, C.SDL_SensorType(sensorType), nil, 0))
	}
	return bool(C.SDL_GetGamepadSensorData(g.cGamepad, C.SDL_SensorType(sensorType), (*C.float)(unsafe.Pointer(&values[0])), C.int(len(values))))
}

// Rumble starts a rumble effect on a gamepad.
func (g *Gamepad) Rumble(lowFrequencyRumble, highFrequencyRumble uint16, durationMs uint32) bool {
	if g == nil || g.cGamepad == nil {
		return false
	}
	return bool(C.SDL_RumbleGamepad(g.cGamepad, C.Uint16(lowFrequencyRumble), C.Uint16(highFrequencyRumble), C.Uint32(durationMs)))
}

// RumbleTriggers starts a rumble effect in a gamepad's triggers.
func (g *Gamepad) RumbleTriggers(leftRumble, rightRumble uint16, durationMs uint32) bool {
	if g == nil || g.cGamepad == nil {
		return false
	}
	return bool(C.SDL_RumbleGamepadTriggers(g.cGamepad, C.Uint16(leftRumble), C.Uint16(rightRumble), C.Uint32(durationMs)))
}

// SetLED updates a gamepad's LED color.
func (g *Gamepad) SetLED(red, green, blue uint8) bool {
	if g == nil || g.cGamepad == nil {
		return false
	}
	return bool(C.SDL_SetGamepadLED(g.cGamepad, C.Uint8(red), C.Uint8(green), C.Uint8(blue)))
}

// SendEffect sends a gamepad-specific effect packet.
func (g *Gamepad) SendEffect(data unsafe.Pointer, size int) bool {
	if g == nil || g.cGamepad == nil {
		return false
	}
	return bool(C.SDL_SendGamepadEffect(g.cGamepad, data, C.int(size)))
}

// AppleSFSymbolsNameForButton gets the Apple sfSymbolsName for a gamepad button.
func (g *Gamepad) AppleSFSymbolsNameForButton(button GamepadButton) string {
	if g == nil || g.cGamepad == nil {
		return ""
	}
	cName := C.SDL_GetGamepadAppleSFSymbolsNameForButton(g.cGamepad, C.SDL_GamepadButton(button))
	if cName == nil {
		return ""
	}
	return C.GoString(cName)
}

// AppleSFSymbolsNameForAxis gets the Apple sfSymbolsName for a gamepad axis.
func (g *Gamepad) AppleSFSymbolsNameForAxis(axis GamepadAxis) string {
	if g == nil || g.cGamepad == nil {
		return ""
	}
	cName := C.SDL_GetGamepadAppleSFSymbolsNameForAxis(g.cGamepad, C.SDL_GamepadAxis(axis))
	if cName == nil {
		return ""
	}
	return C.GoString(cName)
}

// GetBindings gets joystick-layer bindings for an opened gamepad.
func (g *Gamepad) GetBindings() ([]GamepadBinding, error) {
	if g == nil || g.cGamepad == nil {
		return nil, &SDLError{eStr: "invalid gamepad handle"}
	}

	var count C.int
	cBindings := C.SDL_GetGamepadBindings(g.cGamepad, &count)
	if cBindings == nil {
		if count == 0 {
			return []GamepadBinding{}, nil
		}
		return nil, GetError()
	}
	defer C.SDL_free(unsafe.Pointer(cBindings))

	cSlice := unsafe.Slice(cBindings, int(count))
	bindings := make([]GamepadBinding, 0, int(count))
	for _, cb := range cSlice {
		if cb == nil {
			continue
		}

		bindings = append(bindings, GamepadBinding{
			InputType:    GamepadBindingType(cb.input_type),
			InputButton:  int(C.gamepad_binding_input_button(cb)),
			InputAxis:    int(C.gamepad_binding_input_axis(cb)),
			InputAxisMin: int(C.gamepad_binding_input_axis_min(cb)),
			InputAxisMax: int(C.gamepad_binding_input_axis_max(cb)),
			InputHat:     int(C.gamepad_binding_input_hat(cb)),
			InputHatMask: int(C.gamepad_binding_input_hat_mask(cb)),

			OutputType:    GamepadBindingType(cb.output_type),
			OutputButton:  GamepadButton(C.gamepad_binding_output_button(cb)),
			OutputAxis:    GamepadAxis(C.gamepad_binding_output_axis(cb)),
			OutputAxisMin: int(C.gamepad_binding_output_axis_min(cb)),
			OutputAxisMax: int(C.gamepad_binding_output_axis_max(cb)),
		})
	}

	return bindings, nil
}
