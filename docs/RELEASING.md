# Releasing YuhangAlisting

Desktop builds are produced by `.github/workflows/alist-desktop.yml`.

## Manual Build Artifacts

Use **Actions -> YuhangAlisting -> Run workflow** to build packages without a
tag. Manual builds upload workflow artifacts for Windows, macOS, and Linux.
They do not create a GitHub Release.

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
