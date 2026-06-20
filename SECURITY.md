# Security Policy

## Supported Versions

Only the latest released version is actively supported with security fixes.

| Version | Supported |
| ------- | --------- |
| Latest  | Yes       |
| Older   | No        |

## Reporting a Vulnerability

Please do not open a public issue for vulnerabilities.

Report security problems through GitHub's private vulnerability reporting if it
is available for the repository. If that is not available, open a minimal issue
asking for a private contact channel without including exploit details.

Useful details include:

- App version and operating system.
- Whether the issue affects AList management, Rclone mounting, bundled
  binaries, update/release artifacts, or the Tauri shell.
- Reproduction steps.
- Expected impact.

## Scope

In scope:

- Bugs in this desktop app's Rust commands, Vue UI, release workflow, or
  sidecar handling.
- Unsafe handling of paths, credentials, logs, or local process execution.

Out of scope:

- Vulnerabilities in upstream AList, Rclone, WinFsp, Tauri, Vue, or Naive UI.
  Please report those to the corresponding upstream project.
