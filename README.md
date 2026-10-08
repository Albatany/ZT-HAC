# ZT-HAC: Zero-Trust Host Access Controller

<img width="1920" height="1080" alt="screenshot_20261008_052259-region" src="https://github.com/user-attachments/assets/af9dc6f1-28eb-4d42-bda1-e11c31b9c76d" />

A lightweight, offline-first desktop security utility for Linux and Windows. Built with Rust and Tauri, it uses the native OS webview to keep binary size and RAM usage low.

## Features

- **App Restrictor:** block selected applications until you unlock them from the dashboard.
- **Folder Vault:** encrypt and decrypt folders with AES-256-GCM and an Argon2id password-derived key. Sealed files use the `.ztv` extension.
- **Environment Guard:** watch sensitive files such as `.env` or `id_rsa` and seal them automatically when your editors close.
- **Port Stopper:** block TCP/UDP ports using `iptables` on Linux or Windows Firewall on Windows.

## Tech Stack

- **Frontend:** React, TypeScript, TailwindCSS
- **Backend:** Rust, Tauri v2
- **Crates:** `aes-gcm`, `argon2`, `sysinfo`, `notify`, `winapi`

## Getting Started

Requirements: [Rust](https://rustup.rs), Node.js 18+, and the [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/).

On Arch-based distros:

```bash
sudo pacman -S webkit2gtk-4.1 base-devel libappindicator-gtk3 librsvg
```

Run in development:

```bash
npm install
npm run tauri dev
```

Build a release (AppImage on Linux, NSIS installer on Windows):

```bash
npm run tauri build
```

Replace the placeholder icons with your own logo:

```bash
npx tauri icon path/to/logo.png
```

## Notes

- Your vault password is never stored. If you forget it, encrypted data cannot be recovered.
- The Port Stopper needs elevated privileges: a polkit agent on Linux, or Administrator rights on Windows.
- Build the Windows `.exe` on Windows or in CI.

<img width="1002" height="629" alt="screenshot_20261008_052317-region" src="https://github.com/user-attachments/assets/885e9060-15a9-4bdf-aacd-2baa3257ef39" />

