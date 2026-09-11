# Tagryn installation guide

## Objective

Explain how to download and install the current Tagryn alpha on macOS, Linux, and Windows. Add clear instructions to both READMEs, based on the published release and actual CI artifacts.

## Three approaches

1. **Document the existing distribution (selected).** Explain the source release, usable temporary CI packages, and platform-specific source builds. This provides accurate instructions immediately without claiming installer readiness. Its limits are GitHub authentication and artifact expiry for CI downloads, plus a development toolchain where no usable installer is published.
2. **Publish unsigned preview packages.** Package verified builds as permanent release assets with checksums and clear preview labels. This improves downloads but requires a separate packaging and installation verification change, including macOS permissions, signatures, and nested runtime files.
3. **Publish signed installer releases.** Complete macOS signing/notarization, Windows signing, permanent Linux packages, and clean-machine checks. This offers the simplest user experience but requires signing identities and broader release work.

## Work

1. Inspect the release assets, successful CI run, artifact contents, package configuration, and existing documentation.
2. Verify relevant installation and artifact behavior against primary Tauri, GitHub, and platform documentation.
3. Add a prominent download/install section to the English and German READMEs. Distinguish ready-to-run CI packages from source archives; give exact commands and locations for native builds and installation.
4. Check links, commands against the repository scripts and inspected artifacts, Markdown formatting, and the final diff. Do not run a full local test suite for documentation changes.
5. Commit and push the documentation branch, open a pull request to `develop`, and request the maintainer's required merge permission only after the change is reviewable.

## Verification limits

Existing CI checks establish successful builds and tests for the released commit. Inspecting an installer archive is not a clean-machine installation test. This change will report that distinction and will not claim signing, notarization, or support for unbuilt architectures.

## Findings and focused verification

The published release has no binary assets. Successful run `34597693265` contains the Windows x64 NSIS installer `Tagryn_0.1.0_x64-setup.exe` and Linux package `Tagryn_0.1.0_amd64.deb`, both inside the paths now documented. The inspected Debian control file identifies package `tagryn`, architecture `amd64`, and GTK/WebKitGTK runtime dependencies. Artifact metadata gives an expiry of September 25, 2026.

The macOS ARM64 artifact extracts with executable permissions, so permission loss was not assumed from generic artifact documentation. Its bundle fails `codesign --verify --deep --strict` with `code has no resources but signature indicates they must be present`. The existing `scripts/verify-bundle.mjs --adhoc-sign` succeeded on an isolated copy of that artifact in the documentation worktree: 53 Mach-O files, macOS 13.3 deployment limits, no unbundled absolute library dependencies, and ExifTool 13.59 without a system PATH. The READMEs therefore document local macOS building and verification, rather than presenting the CI app as a verified installer.

The installed Tauri CLI confirms `--no-bundle`. Its locked resource-resolution implementation uses resources next to executables in Cargo output directories, which supports the documented local Linux executable path while keeping the build directory intact.

Primary references: [GitHub artifact downloads](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/download-workflow-artifacts), [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/), [Tauri Windows installers](https://v2.tauri.app/distribute/windows-installer/), and [Tauri Debian packages](https://v2.tauri.app/distribute/debian/).

Both READMEs passed local relative-link and heading-anchor checks, shell-fence syntax checks, npm-script reference checks, Prettier, and `git diff --check`. The linked successful release build is readable through the unauthenticated GitHub API. This documentation-only change does not repeat the unchanged application test/build matrix; the release commit remains the tested application baseline.

## Unresolved questions

- May the finished documentation pull request be merged into `develop`? Explicit maintainer permission is required before any merge.
- Should a later release add permanent preview downloads or proceed directly to signed installers? This does not block the current documentation.
