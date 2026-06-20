# Releasing YuhangAlisting

This project uses GitHub Actions to build desktop installers.

## Normal Release

1. Update the version in:
   - `alist-desktop/package.json`
   - `alist-desktop/src-tauri/tauri.conf.json`
   - `alist-desktop/src-tauri/Cargo.toml`

2. Commit the version bump:

   ```bash
   git add .
   git commit -m "chore: release v0.1.0"
   ```

3. Create and push a version tag:

   ```bash
   git tag v0.1.0
   git push origin main
   git push origin v0.1.0
   ```

4. GitHub Actions runs the `release` job for:
   - `windows-latest`
   - `macos-latest`
   - `ubuntu-22.04`

5. The workflow creates a draft GitHub Release and uploads the generated
   installers. Review the draft release notes, then publish it.

## Manual Build Artifacts

Use **Actions -> AList Desktop -> Run workflow** to build installers without a
tag. Manual builds upload workflow artifacts only; they do not create a GitHub
Release.

## Local Release Smoke Test

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
