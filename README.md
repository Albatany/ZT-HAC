# ZT-HAC (Zero-Trust Host Access Controller)

Requirements: Rust, Node 18+, Tauri v2 system deps (Arch/CachyOS: `sudo pacman -S webkit2gtk-4.1 base-devel libappindicator-gtk3 librsvg`).

    npm install
    npm run tauri dev        # development
    npm run tauri build      # release (AppImage on Linux, NSIS .exe on Windows)

Icons are placeholders. Replace with: `npx tauri icon path/to/logo.png`.
Windows .exe should be built on Windows (or CI); cross-compiling from Linux is not set up here.
