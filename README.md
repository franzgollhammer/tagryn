<p align="center"><img src="assets/brand/wordmark.svg" alt="Tagryn" width="260"></p>

# Your files. Every detail.

An offline desktop workbench for inspecting, comparing, and deliberately editing file metadata. Built with Tauri, Rust, Vue, and ExifTool. Your files stay on your machine. No account, telemetry, or automatic map requests.

[Deutsch](README.de.md) · [Contribute](CONTRIBUTING.md) · [Roadmap](docs/ROADMAP.md) · [Discussions](https://github.com/franzgollhammer/tagryn/discussions) · [Releases](https://github.com/franzgollhammer/tagryn/releases)

[![Verify Tagryn](https://github.com/franzgollhammer/tagryn/actions/workflows/verify.yml/badge.svg?branch=develop)](https://github.com/franzgollhammer/tagryn/actions/workflows/verify.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

![Tagryn desktop app with synthetic image fixtures, light theme](docs/images/workspace-light.png)

_The native macOS app showing generated test images. The interface supports English and German._

## Source alpha

Tagryn 0.1.0 is working early software with real file access, write plans, backups, and restoration. The first distribution is a **source alpha**; supported signed installers are a later milestone. See the [verification runs](https://github.com/franzgollhammer/tagryn/actions/workflows/verify.yml) for current CI results and the [release notes](docs/releases/v0.1.0-alpha.1.md) for limitations.

Start with disposable copies and keep independent backups. Tagryn does not promise complete anonymization. Original metadata can remain in backups, the local cache, job history, and exports. Read the [data-integrity limits](docs/SAFETY.md) before working with important files.

## What you can do

- **Inspect:** open files or folders, scan recursively, browse a virtualized list or thumbnail grid, search loaded metadata, and inspect raw values with their exact tag identities.
- **Compare:** select up to eight files, see metadata differences, and use custom columns, favorites, and saved views.
- **Edit deliberately:** stage supported field edits, keywords, dates, GPS data, XMP sidecars, presets, metadata transfers, renaming, and CSV/JSON imports. Review the actual plan before saving.
- **Verify and restore:** create checked backups, reread saved values, inspect per-file job results, and restore only after conflict checks.
- **Work comfortably:** light/dark/system themes, adjustable text and panes, keyboard navigation, and a command palette.

Read support and write support differ by format. RAW edits use XMP sidecars; embedded audio/video/PDF writes, arbitrary technical tag editing, GPX matching, online maps, live filesystem watching, and automatic updates are not implemented. See the [format rules](docs/FORMATS.md).

## Build from source

Install Node.js 24 LTS, npm, Rust through rustup, and the [Tauri system prerequisites](https://v2.tauri.app/start/prerequisites/). The repository pins Rust in `rust-toolchain.toml` and dependencies in the lockfiles.

| Platform    | Additional requirements                                                                                   |
| ----------- | --------------------------------------------------------------------------------------------------------- |
| macOS       | macOS 13.3 or newer; Xcode Command Line Tools. Apple Silicon and Intel have separate CI builds.           |
| Windows x64 | Visual Studio Build Tools with Desktop development with C++, plus WebView2.                               |
| Linux x64   | Compiler tools and GTK/WebKitGTK 4.1 development libraries. The workflow specifies Ubuntu 22.04 packages. |

```sh
git clone https://github.com/franzgollhammer/tagryn.git
cd tagryn
npm ci
npm run runtime
npm run icons
npm run licenses
npm run desktop
```

The first runtime build downloads checksum-pinned packages and compiles private Perl on macOS/Linux. It needs an internet connection and a C compiler. Later app launches and core workflows are offline; no existing ExifTool or Perl installation is required.

`npm run dev` runs the frontend only. Native file access and saving require `npm run desktop`. The browser review mode is read-only, uses generated fixtures, and is excluded from production builds. Use `TAGRYN_PROFILE_DIR` with an absolute path when you need an isolated development profile.

Platform packaging commands and the first save/restore walkthrough are in the [detailed German guide](README.de.md). A compiled app does not need Node.js or Rust. Local builds and CI artifacts are not signed installer releases.

## Build Tagryn with us

**AI-assisted contributions are welcome.** Use the tools you prefer to investigate bugs, write code, improve docs, and review changes. Understand what you submit, verify the behavior, and remain available for review. Small, clear pull requests make this work.

Start with [CONTRIBUTING.md](CONTRIBUTING.md). Agents should read [AGENTS.md](AGENTS.md). English documentation, synthetic format fixtures, accessibility checks, and focused bug fixes are useful first contributions. The complete test suite runs in GitHub Actions; local checks stay focused on the current change.

Franz Gollhammer maintains the project and its direction. Propose larger changes in an issue first. [Project principles](docs/PRINCIPLES.md) explain what belongs in Tagryn, and the [roadmap](docs/ROADMAP.md) lists priorities without promising delivery dates.

## Documentation

- [Architecture and runtime distribution](docs/ARCHITECTURE.md) — Rust service, metadata identity, cache, and bundling.
- [Formats and synchronization](docs/FORMATS.md) — what can be read and written.
- [Data integrity](docs/SAFETY.md) — write protocol, backup, restore, and known boundaries.
- [Historical local validation](docs/VALIDATION.md) — measurements and checks from the initial development build.
- [Release process](docs/RELEASING.md) and [private security reports](SECURITY.md).

The detailed technical documents are currently in German. English translations are welcome.

## License and credits

Tagryn's original code and documentation are released under the [MIT license](LICENSE). Third-party code and bundled runtimes retain their own licenses; see [third-party notices](docs/THIRD-PARTY-NOTICES.md) and [dependency license texts](docs/DEPENDENCY-LICENSES.txt).

ExifTool is by Phil Harvey. Tagryn also builds on Perl, Tauri, Vue, and the other components listed in its lockfiles. Omarchy's [doctrine](https://omarchy.org/doctrine/) inspired the approach to opinionated open source software and welcoming agents. Tagryn is independently maintained by [Franz Gollhammer](https://github.com/franzgollhammer).
