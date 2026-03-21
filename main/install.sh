#!/usr/bin/env sh

set -e

SISR_VERSION="dev-snapshot"
VIIPER_VERSION="dev-snapshot"

REPO="Alia5/SISR"
API_URL="https://api.github.com/repos/${REPO}/releases/latest"

if [ "$SISR_VERSION" = "dev-snapshot" ]; then
    API_URL="https://api.github.com/repos/${REPO}/releases/tags/dev-snapshot"
elif echo "$SISR_VERSION" | grep -qE '^v?[0-9]+\.[0-9]+'; then
    API_URL="https://api.github.com/repos/${REPO}/releases/tags/${SISR_VERSION}"
fi

echo "Fetching SISR release: $SISR_VERSION..."
RELEASE_DATA=$(curl -fsSL "$API_URL")
VERSION=$(printf '%s' "$RELEASE_DATA" \
    | grep -Eo '"tag_name"[[:space:]]*:[[:space:]]*"[^"]+"' \
    | head -n 1 \
    | cut -d'"' -f4)

if [ -z "$VERSION" ]; then
    echo "Error: Could not fetch SISR release" 
    exit 1
fi

echo "Version: $VERSION"

ARCH=$(uname -m)

case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *)
        echo "Error: Unsupported architecture: $ARCH" 
        echo "Supported: x86_64, aarch64" 
        exit 1
        ;;
esac

ASSET_NAME="SISR-${ARCH}.AppImage"

echo "Architecture: $ARCH"
echo "Looking for asset: $ASSET_NAME"

DOWNLOAD_URL=$(printf '%s' "$RELEASE_DATA" \
    | grep -Eo '"browser_download_url"[[:space:]]*:[[:space:]]*"[^"]+"' \
    | grep "$ASSET_NAME" \
    | head -n 1 \
    | cut -d'"' -f4)

if [ -z "$DOWNLOAD_URL" ]; then
    echo "Error: Could not find asset $ASSET_NAME" 
    exit 1
fi

echo "Downloading from: $DOWNLOAD_URL"
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

cd "$TEMP_DIR"
if ! curl -fsSL -o SISR.AppImage "$DOWNLOAD_URL"; then
    echo "Error: Could not download SISR AppImage" 
    exit 1
fi

chmod +x SISR.AppImage

INSTALL_DIR="$HOME/.local/share/SISR"
IS_UPDATE=0
if [ -f "$INSTALL_DIR/SISR.AppImage" ]; then
    IS_UPDATE=1
    echo "Existing SISR installation detected"
fi

echo "Installing to $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"
cp SISR.AppImage "$INSTALL_DIR/SISR.AppImage"
chmod +x "$INSTALL_DIR/SISR.AppImage"
echo "Installed SISR AppImage"

IS_STEAMOS=0
WAS_READONLY=0

if [ -f /etc/os-release ]; then
    if grep -qi '^ID=steamos' /etc/os-release; then
        IS_STEAMOS=1
        echo "SteamOS detected"
        
        if command -v steamos-readonly >/dev/null ; then
            if steamos-readonly status | grep -q "enabled"; then
                WAS_READONLY=1
                echo "Read-only root is enabled. Temporarily disabling..."
                if steamos-readonly disable; then
                    echo "Read-only root disabled"
                else
                    echo "Warning: Could not disable read-only root. Some operations may fail." 
                fi
            else
                echo "Read-only root is already disabled"
            fi
        fi
    fi
fi

echo ""
echo "Installing VIIPER..."

echo "Installing VIIPER version: $VIIPER_VERSION"
VIIPER_INSTALL_VERSION=$VIIPER_VERSION
if [ "$VIIPER_INSTALL_VERSION" = "dev-snapshot" ]; then
    VIIPER_INSTALL_VERSION="main"
fi
if curl -fsSL "https://alia5.github.io/VIIPER/${VIIPER_INSTALL_VERSION}/install.sh" | sh; then
    echo "VIIPER installed successfully"
else
    echo "Warning: VIIPER installation failed. You may need to install it manually." 
    echo "See: https://alia5.github.io/VIIPER/stable/getting-started/installation/" 
fi

echo ""
echo "Configuring Steam CEF remote debugging..."

CEF_CREATED=0

STEAM_PATH="$HOME/.steam/steam"
if [ -d "$STEAM_PATH" ]; then
    CEF_FILE="$STEAM_PATH/.cef-enable-remote-debugging"
    if touch "$CEF_FILE" 2>/dev/null; then
        echo "Created CEF debug file in: $STEAM_PATH"
        CEF_CREATED=1
    else
        echo "Warning: Could not create CEF debug file in $STEAM_PATH" 
    fi
fi

FLATPAK_STEAM_PATH="$HOME/.var/app/com.valvesoftware.Steam/data/Steam"
if [ -d "$FLATPAK_STEAM_PATH" ]; then
    CEF_FILE="$FLATPAK_STEAM_PATH/.cef-enable-remote-debugging"
    if touch "$CEF_FILE" 2>/dev/null; then
        echo "Created CEF debug file in: $FLATPAK_STEAM_PATH"
        CEF_CREATED=1
    else
        echo "Warning: Could not create CEF debug file in $FLATPAK_STEAM_PATH" 
    fi
fi

if [ $CEF_CREATED -eq 0 ]; then
    echo "Warning: Could not find Steam installation or create CEF debug file" 
    echo "You may need to manually create .cef-enable-remote-debugging in your Steam directory" 
fi

echo ""
echo "Creating desktop entry..."

DESKTOP_FILE="$HOME/.local/share/applications/sisr.desktop"
ICON_FILE="$INSTALL_DIR/SISR.svg"
mkdir -p "$(dirname "$DESKTOP_FILE")"

echo "Downloading SISR icon..."
if curl -fsSL -o "$ICON_FILE" "https://raw.githubusercontent.com/${REPO}/main/docs/SISR.svg"; then
    echo "Icon downloaded successfully"
else
    echo "Warning: Could not download icon" 
fi

cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Name=SISR
Comment=Steam Input System Redirector
Exec=$INSTALL_DIR/SISR.AppImage
Icon=$ICON_FILE
Terminal=false
Categories=Game;Utility;
EOF

if [ $? -eq 0 ]; then
    echo "Created desktop entry: $DESKTOP_FILE"
    
    if command -v update-desktop-database >/dev/null ; then
        update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
    fi
else
    echo "Warning: Could not create desktop entry" 
fi

if [ $IS_STEAMOS -eq 1 ] && [ $WAS_READONLY -eq 1 ]; then
    echo ""
    echo "Re-enabling SteamOS read-only root..."
    if steamos-readonly enable; then
        echo "Read-only root re-enabled"
    else
        echo "Warning: Could not re-enable read-only root. You may need to do this manually." 
    fi
fi

echo ""
echo "================================================"
echo "SISR installed successfully!"
echo "================================================"
echo ""
echo "Installation location: $INSTALL_DIR"
echo "Executable: $INSTALL_DIR/SISR.AppImage"
echo ""
echo "You can now run SISR from your application menu or by running:"
echo "  $INSTALL_DIR/SISR.AppImage"
echo ""

if [ $IS_UPDATE -eq 1 ]; then
    echo "Update complete!"
fi
