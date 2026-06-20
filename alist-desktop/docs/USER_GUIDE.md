# AList Desktop User Guide

## First Run

1. Start the application.
2. Open **Dashboard** and check **安装前检查**. It shows whether AList,
   Rclone, and WinFsp are ready.
3. Open **Settings** and confirm the AList port. The default is `5244`.
4. Put sidecar binaries in `binaries/`, or run:

```bash
yarn prebuild
```

5. Open **Dashboard** and click **启动服务**.
6. Click **打开网页** to enter the AList web console.
7. Add mount configs in **Mounts**, then click **挂载** and enter the AList
   admin password in the app dialog.

## AList Service

Dashboard shows the current service status, port, uptime, data directory, binary
path, and restart attempts. If the AList process exits unexpectedly, the app
tries to restart it up to three times.

Available actions:

- Start, stop, and restart AList.
- Open the web console.
- Reset the admin password and copy the new value.

AList prints the first startup password only once. After that, the password is
stored as a hash and cannot be revealed. Use reset or set a known password.

## Rclone Mounts

Open **Mounts** to create WebDAV mounts backed by AList.

Required fields:

- Name: display name in the desktop app.
- AList path: remote path such as `/aliyundrive`.
- Local path: `Z:` on Windows, `/Volumes/AList` on macOS, or `/mnt/alist` on Linux.
- Cache mode: `off`, `minimal`, `writes`, or `full`.

Windows mounting requires WinFsp. Linux mounting requires FUSE permissions.
The app asks for the AList admin password before mounting and automatically
passes an obscured password to Rclone.

## Logs

Open **Logs** to watch AList and Rclone process output. Logs support:

- Source filtering.
- Level filtering.
- Text search.
- Auto-scroll.
- Clear local log buffer.

## Settings

Settings are stored in the app data directory as `config.json`.

Available settings:

- AList port.
- Start AList when the app opens.
- Mount auto-enabled Rclone configs when the app opens.
- Start the app when the operating system logs in.
- Theme: system, light, dark.
- Language: Chinese or English for core shell labels.
- Custom AList and Rclone binary paths.
- Update checks against GitHub releases.

## Packaging

Release builds should download sidecar binaries before bundling:

```bash
yarn prebuild --force
yarn tauri build
```

GitHub Actions can build Windows NSIS/MSI, macOS DMG, and Linux AppImage/deb
artifacts from the workflow in `.github/workflows/alist-desktop.yml`.
