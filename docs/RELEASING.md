# Releasing YuhangAlisting

Desktop builds are produced by `.github/workflows/alist-desktop.yml`.

## Manual Build Artifacts

Use **Actions -> YuhangAlisting -> Run workflow** to build packages without a
tag. Manual builds upload workflow artifacts for Windows, macOS, and Linux.
They do not create a GitHub Release.

Manual artifacts are useful for smoke testing before publishing:

- `yuhang-alisting-windows-latest`: NSIS `.exe`, MSI `.msi`, and bundled sidecars.
- `yuhang-alisting-macos-latest`: `.app`, `.app.zip`, and `.dmg`.
- `yuhang-alisting-ubuntu-22.04`: AppImage and Debian `.deb`.

The macOS `.app` bundle is ad-hoc signed in CI so it can be opened for local
testing, but it is not notarized by Apple.

## Tagged GitHub Release

1. Update the version in:
   - `alist-desktop/package.json`
   - `alist-desktop/src-tauri/tauri.conf.json`
   - `alist-desktop/src-tauri/Cargo.toml`

2. Commit the version bump:

   ```bash
   git add .
   git commit -m "chore: release v0.1.0"
   ```

3. Create and push a semver tag:

   ```bash
   git tag v0.1.0
   git push origin main
   git push origin v0.1.0
   ```

4. GitHub Actions creates a draft GitHub Release with generated notes and
   installer assets.

5. Review the draft release, test the uploaded packages, then publish it.

## Local Smoke Test

```bash
cd alist-desktop
yarn install --immutable
yarn prebuild --force
yarn build
cargo check --manifest-path src-tauri/Cargo.toml
yarn tauri build --ci
```

Generated bundles are written to:

```text
alist-desktop/src-tauri/target/release/bundle/
```

## Smoke Test Checklist

- Start AList from the dashboard and confirm `/ping` health checks succeed.
- Open the AList web UI and log in with `admin / root` on a fresh data dir.
- Change the admin password and confirm it is not reset after restarting.
- Add an Rclone WebDAV mount configuration and verify mount/unmount behavior.
- On Windows, verify WinFsp detection and installer launch.
- On macOS, confirm the app opens without the legacy Carbon window controls.
