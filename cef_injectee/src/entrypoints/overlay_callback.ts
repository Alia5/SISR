import { api } from "../lib/api";
const main = async () => {
    const overlayCallback = (
        some_number: number,
        always_0: number,
        overlay_opened_closed_bool: boolean,
        always_true: boolean
    ) => {
        api.overlayStateChanged(overlay_opened_closed_bool);
    }

    // Injected into "normal" overlay-tab
    if (!!opener && (opener as SteamWindow)?.SteamClient?.Overlay) {
        await (opener as SteamWindow).SteamClient.Overlay.RegisterForOverlayActivated(
            overlayCallback
        );
    } else {
        // Injected into "Gaming Mode", no overlay tab exists,
        // but we can query focus of the big picture menu ;)
        window.addEventListener("focus", () => {
            api.overlayStateChanged(true);
        });
        window.addEventListener("focusout", () => {
            api.overlayStateChanged(false);
        });
    }

};

api.connect().then(() => {
    main().catch(console.error);
}).catch(console.error);