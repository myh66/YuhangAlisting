# Contributing

Thanks for taking the time to improve YuhangAlisting.

## Development Setup

```bash
git clone https://github.com/myh66/YuhangAlisting.git
cd YuhangAlisting/alist-desktop
corepack enable
yarn install --immutable
yarn prebuild
yarn tauri dev
```

## Before Opening a Pull Request

Please run the same checks used by CI:

```bash
cd alist-desktop
yarn build
cargo check --manifest-path src-tauri/Cargo.toml
```

For release-related changes, also run:

```bash
yarn tauri build --debug --no-bundle --ci
```

## Pull Request Guidelines

- Keep changes focused; avoid bundling unrelated refactors with feature work.
- Include screenshots or short recordings for UI changes.
- Explain user-visible behavior changes in the PR description.
- Do not commit downloaded sidecar binaries from `alist-desktop/binaries/`.
- Update README or docs when setup, release, or user workflow changes.

## Commit Style

Use concise messages such as:

```text
feat: add WinFsp installer check
fix: handle AList startup config
docs: clarify release flow
```

## Reporting Bugs

Use the bug report template and include:

- Operating system and version.
- App version.
- Whether AList, Rclone, and WinFsp are detected in the dashboard.
- Relevant logs from the Logs page.
