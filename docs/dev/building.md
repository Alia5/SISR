# 🏗️ Building

## Prerequisites

- Rust toolchain  
  [rustup.rs](https://rustup.rs/)
- CMake and Ninja  
  SDL3 is built from source via CMake w/ Ninja
- Node.js  
  Required to build the UI (`UI/`) and the CEF payloads (`CEF_Payloads/`) that
  interact with Steam's CEF remote debugging interface

!!! info "SDL3"
    SDL3 is compiled from source via `sdl3-sys`. `build.rs` sets `CMAKE_GENERATOR=Ninja`.

### 🪟 Windows

- MSVC toolchain
- Ensure `cmake` and `ninja` are in PATH

### 🐧 Linux

#### 🏹 Arch Linux

```bash
sudo pacman -S ninja cmake pkg-config gtk3 webkit2gtk-4.1 xdotool libxss
```

#### 🟠 Ubuntu

```bash
sudo apt-get install ninja-build cmake pkg-config libgtk-3-dev \
  libsoup-3.0-dev libjavascriptcoregtk-4.1-dev libwebkit2gtk-4.1-dev \
  libxdo-dev libxss-dev
```

## Build

Build the CEF payloads and the Web UI first, then build SISR.

**1. CEF Payloads** (Steam overlay injection scripts):

```bash
cd CEF_Payloads
npm install
npm run build
cd ..
```

**2. Web UI** (SvelteKit frontend served inside the app window):

```bash
cd UI
npm install
npm run build
cd ..
```

**3. SISR**:

```bash
cargo build
# or for a release build:
cargo build --release
```

!!! info "VIIPER"
    VIIPER is downloaded at build time based on `package.metadata.viiper` in `Cargo.toml`.  
    Binary is placed in `target/<triple>/<profile>/viiper(.exe)`.  
    Internet required on first build; subsequent builds use cached copy.
