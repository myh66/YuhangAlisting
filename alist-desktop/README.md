# AList Desktop

Tauri 2 + Vue 3 + TypeScript desktop client for managing local AList and Rclone
sidecar services.

## Features

- AList process lifecycle: start, stop, restart, health check, crash restart.
- AList admin password read, reset, and set commands.
- Rclone WebDAV mount CRUD with local JSON persistence.
- Windows drive-letter mounts and Linux/macOS directory mounts.
- Real-time stdout/stderr log capture through Tauri events.
- Tray icon with show, hide, start, stop, and quit actions.
- Close-to-tray behavior.
- Settings page with port, startup, theme, language, binary paths, and update checks.
- GitHub Actions release workflow for Windows NSIS/MSI, macOS DMG, Linux AppImage/deb.

## Development

```bash
yarn
yarn tauri dev
```

If Yarn is not available as a global command, use Corepack directly:

```bash
corepack yarn tauri dev
```

## Sidecar Binaries

The app expects AList and Rclone binaries in `binaries/`.

```bash
yarn prebuild
```

The script detects the current platform, downloads matching archives, extracts
the executables, and writes them to `binaries/alist` and `binaries/rclone`
(`.exe` on Windows).

## Verification

```bash
yarn build
cargo check --manifest-path src-tauri/Cargo.toml
yarn tauri build --debug --no-bundle --ci
```

## Documentation

See `docs/USER_GUIDE.md`.
