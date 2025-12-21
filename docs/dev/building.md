# 🏗️ Building

## Prerequisites

- Rust toolchain  
  [rustup.rs](https://rustup.rs/)
- CMake and Ninja  
  SDL3 is built from source via CMake w/ Ninja
- Node.js
  Required to build the javascript parts interacting with Steams CEF remote debugging interface
- pnpm  
  is just better than npm  
  [pnpm.io](https://pnpm.io/installation)

!!! info "SDL3"
    SDL3 is compiled from source via `sdl3-sys`. `build.rs` sets `CMAKE_GENERATOR=Ninja`.

### 🪟 Windows

- MSVC toolchain
- Ensure `cmake` and `ninja` are in PATH

### 🐧 Linux

#### 🏹 Arch Linux

```bash
sudo pacman -S ninja cmake pkg-config gtk3 xdotool
```

#### 🟠 Ubuntu

```bash
sudo apt-get install ninja-build cmake pkg-config libgtk-3-dev libxdo-dev
```

## Build

Build the CEF injectee first, then build SISR

```bash
cd cef_injectee
pnpm install
pnpm run build
cd ..
```

Then build SISR:

```bash
cargo build
# or
cargo build --release
```

!!! info "VIIPER"
    VIIPER is downloaded at build time based on `package.metadata.viiper` in `Cargo.toml`.  
    Binary is placed in `target/<triple>/<profile>/viiper(.exe)`.  
    Internet required on first build; subsequent builds use cached copy.
