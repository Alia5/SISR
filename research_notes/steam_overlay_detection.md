# Steam Overlay Detection - The Right Way

- `BOverlayNeedsPresent` from GameOverlayRenderer DLL is useless - returns true even for FPS counter updates


## Solution: CEF Debugging Protocol

### Steps

1. **Ensure CEF Debugging is enabled**
   - Steam launch option or config setting needed

2. **Connect to CEF debug protocol**
   - Find the debug port
   - Wait for a tab named `"SP Overlay: someId/WHatever/always_0_or_what"` or similar (contains `"SP Overlay"`)

3. **Inject JavaScript callback**
   ```javascript
   await opener.SteamClient.Overlay.RegisterForOverlayActivated(
     (some_number, always_0, overlay_opened_closed_bool, always_true) => 
       console.log("overlay_change_detected", overlay_opened_closed_bool)
   );
   ```

4. **Bridge JS callback to Rust**
   - Get back to Rust code somehow (e.g., WebSocket, HTTP server, etc.)

5. **PROFIT!** `1337 h4xx0r 5h17`
