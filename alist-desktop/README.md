# AList Desktop App

This is the Tauri 2 application package for YuhangAlisting.

## Scripts

```bash
yarn install --immutable
yarn prebuild
yarn tauri dev
```

Useful checks:

```bash
yarn build
cargo check --manifest-path src-tauri/Cargo.toml
yarn tauri build --debug --no-bundle --ci
```

## Layout

```text
alist-desktop/
├─ binaries/          # generated AList/Rclone/WinFsp sidecars
├─ docs/              # app documentation
├─ scripts/           # build preparation scripts
├─ src/               # Vue frontend
└─ src-tauri/         # Rust backend, commands, services, tray
```

The generated binaries are intentionally ignored. Run `yarn prebuild` before
local development or release builds.
