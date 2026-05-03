
declare global {

    interface SteamWindow extends Window {
        SteamClient: SteamClient;
    }

    interface Window {
        SteamClient?: SteamClient;
        // @ts-expect-error ignore
        opener?: SteamWindow | null;
        __sisrCleanup?: {
            [key: string]: (() => void);
        };
        __SISR_API_URL: string;

        appStore: {
            allApps: {
                appid: number;
                app_type: number;
                installed: boolean;
                [key: string]: unknown;
            }[];
        };
        ControllerStore: ControllerStore;
    }

    let __sisrCleanup: { [key: string]: (() => void) } | undefined;
    let __SISR_API_URL: string;


    const SteamClient: SteamClient;
    // @ts-expect-error ignore
    const opener: SteamWindow | null;
    let __INJECT_RETURN: unknown;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const goTmpl: (paramName: string) => any;
    const SteamUIStore: {
        ActiveWindowInstance: {
            m_Navigator: {
                SteamWebTab: (url: string) => void;
            };
        };
    };
}

// Best effort, Edited as needed
interface SteamClient {
    Apps: SteamClientApps;
    Auth: SteamClientAuth;
    Broadcast: Record<string, (...args: unknown[]) => unknown>;
    Browser: SteamClientBrowser;
    BrowserView: {
        Create: (...args: unknown[]) => void;
        CreatePopup: (...args: unknown[]) => void;
        Destroy: (id: number) => void;
        PostMessageToParent: (message: unknown) => void;
    };
    ClientNotifications: Record<string, (...args: unknown[]) => unknown>;
    Cloud: Record<string, (...args: unknown[]) => unknown>;
    CloudStorage: {
        WriteKey: (key: string, value: unknown) => void;
    };
    CommunityItems: Record<string, (...args: unknown[]) => unknown>;
    Compat: Record<string, (...args: unknown[]) => unknown>;
    Console: Record<string, (...args: unknown[]) => unknown>;
    Customization: Record<string, (...args: unknown[]) => unknown>;
    Downloads: Record<string, (...args: unknown[]) => unknown>;
    FamilySharing: Record<string, (...args: unknown[]) => unknown>;
    FriendSettings: Record<string, (...args: unknown[]) => unknown>;
    Friends: Record<string, (...args: unknown[]) => unknown>;
    GameNotes: Record<string, (...args: unknown[]) => unknown>;
    GameRecording: Record<string, (...args: unknown[]) => unknown>;
    GameSessions: Record<string, (...args: unknown[]) => unknown>;
    Input: Record<string, (...args: unknown[]) => unknown>;
    InstallFolder: Record<string, (...args: unknown[]) => unknown>;
    Installs: Record<string, (...args: unknown[]) => unknown>;
    MachineStorage: {
        GetString: (key: string) => string | undefined;
        SetString: (key: string, value: string) => void;
        GetJSON: (key: string) => unknown;
        SetObject: (key: string, value: unknown) => void;
        DeleteKey: (key: string) => void;
    };
    Messaging: Record<string, (...args: unknown[]) => unknown>;
    Music: Record<string, (...args: unknown[]) => unknown>;
    Notifications: {
        RegisterForNotifications: (callback: (notification: unknown) => void) => void;
    };
    OpenVR: Record<string, (...args: unknown[]) => unknown>;
    Overlay: SteamClientOverlay;
    Parental: Record<string, (...args: unknown[]) => unknown>;
    RemotePlay: Record<string, (...args: unknown[]) => unknown>;
    RoamingStorage: {
        GetString: (key: string) => string | undefined;
        SetString: (key: string, value: string) => void;
        GetJSON: (key: string) => unknown;
        SetObject: (key: string, value: unknown) => void;
        DeleteKey: (key: string) => void;
    };
    Screenshots: Record<string, (...args: unknown[]) => unknown>;
    ServerBrowser: Record<string, (...args: unknown[]) => unknown>;
    Settings: SteamClientSettings;
    SharedConnection: Record<string, (...args: unknown[]) => unknown>;
    Stats: {
        RecordDisplayEvent: (event: string) => void;
        RecordActivationEvent: (event: string) => void;
    };
    SteamChina: Record<string, (...args: unknown[]) => unknown>;
    Storage: {
        GetString: (key: string) => string | undefined;
        SetString: (key: string, value: string) => void;
        GetJSON: (key: string) => unknown;
        SetObject: (key: string, value: unknown) => void;
        DeleteKey: (key: string) => void;
    };
    Streaming: Record<string, (...args: unknown[]) => unknown>;
    System: SteamClientSystem;
    UI: SteamClientUI;
    URL: Record<string, (...args: unknown[]) => unknown>;
    Updates: Record<string, (...args: unknown[]) => unknown>;
    User: SteamClientUser;
    WebChat: Record<string, (...args: unknown[]) => unknown>;
    WebUITransport: {
        GetTransportInfo: () => unknown;
        NotifyTransportFailure: () => void;
    };
    Window: SteamClientWindow;
    _internal: Record<string, (...args: unknown[]) => unknown>;
}

interface SteamClientApps {
    RunGame: (appId: number, ...args: unknown[]) => void;
    VerifyApp: (appId: number) => void;
    StreamGame: (appId: number) => void;
    CancelLaunch: (appId: number) => void;
    TerminateApp: (appId: number) => void;
    AddShortcut(appName: string, executablePath: string, directory: string, launchOptions: string): Promise<number>;
    SetShortcutExe(appId: number, path: string): void;
    SetShortcutIcon(appId: number, path: string): void;
    SetShortcutIsVR(appId: number, value: boolean): void;
    SetShortcutLaunchOptions(appId: number, options: string): void;
    SetShortcutName(appId: number, name: string): void;
    SetShortcutStartDir(appId: number, directory: string): void;
    RegisterForAppDetails(appId: number, callback: (details: Record<string, unknown>) => void): void;
    DownloadWorkshopItem(appId: number, itemId: number, whatever: boolean): Promise<void>;
    ShowControllerConfigurator(appId: number): void;
}

interface SteamClientAuth {
    StartSignInFromCache: (...args: unknown[]) => void;
    ValidateCachedSignInPin: (pin: string) => void;
    ClearCachedSignInPin: () => void;
    SetCachedSignInPin: (pin: string) => void;
    UserHasCachedSignInPin: () => boolean;
}

interface SteamClientBrowser {
    RegisterForOpenNewTab: (callback: (url: string) => void) => void;
    RegisterForGestureEvents: (callback: (...args: unknown[]) => void) => void;
    SetShouldExitSteamOnBrowserClosed: (shouldExit: boolean) => void;
    NotifyUserActivation: () => void;
    GetBrowserID: () => number;
}

interface SteamClientOverlay {
    DestroyGamePadUIDesktopConfiguratorWindow: () => void;
    RegisterOverlayBrowserInfoChanged: (callback: (info: unknown) => void) => void;
    GetOverlayBrowserInfo: () => unknown;
    RegisterForActivateOverlayRequests: (callback: (...args: unknown[]) => void) => void;
    RegisterForOverlayActivated: (
        callback: (
            some_number: number,
            always_0: number,
            overlay_opened_closed_bool: boolean,
            always_true: boolean
        ) => void) => Promise<unknown>;
}

interface SteamClientSystemDisplay {
    [key: string]: unknown;
}

interface SteamClientSystemUI {
    [key: string]: unknown;
}

interface SteamClientSystem {
    UI: SteamClientSystemUI;
    Display: SteamClientSystemDisplay;
    GetSystemInfo: () => {
        nSteamVersion: number;
        sSteamAPI: string;
        sSteamBuildDate: string;
        sOSCodename: string;
        sOSVersionId: string;
        sOSVariantId: string;
        sOSBuildId: string;
        sBIOSVersion: string;
    };
    VideoRecordingDriverCheck: () => void;
    SwitchToDesktop: () => void;
}

interface SteamClientWindow {
    SetGamepadUIManualDisplayScaleFactor: (scale: number) => void;
    SetGamepadUIAutoDisplayScale: () => void;
    Minimize: () => void;
    ProcessShuttingDown: () => void;
    ToggleMaximize: () => void;
}

interface SteamClientUI {
    RegisterDesiredSteamUIWindowsChanged: (callback: (windows: unknown[]) => void) => void;
    GetDesiredSteamUIWindows: () => unknown[];
    EnsureMainWindowCreated: () => void;
    NotifyAppInitialized: () => void;
    GetOSEndOfLifeInfo: () => unknown;
}

interface SteamClientSettings {
    RegisterForSettingsChanges: (callback: (settings: unknown) => void) => void;
    RegisterForSettingsArrayChanges: (callback: (settings: unknown[]) => void) => void;
    SetSetting: (key: string, value: unknown) => void;
    ClearDownloadCache: () => void;
    RegisterForAppsWithAutoUpdateOverrides: (callback: (apps: unknown[]) => void) => void;
}

interface SteamClientUser {
    RegisterForCurrentUserChanges: (callback: (user: unknown) => void) => void;
    AuthorizeMicrotxn: (...args: unknown[]) => void;
    CancelMicrotxn: (...args: unknown[]) => void;
    RequestSupportSystemReport: () => void;
    SetAsyncNotificationEnabled: (enabled: boolean) => void;
}

export { };

interface ControllerStore {
    DoControllerListQuery: () => Promise<void>;
    m_controllerList: unknown[];
    prototype?: ControllerStore;
}
