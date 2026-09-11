# Releasing Tagryn

Releases are maintained by Franz Gollhammer. The integration branch is `develop`; `main` is not used. Obtain explicit maintainer permission before any merge.

## Source alpha

The first source release is planned as `v0.1.0-alpha.1`. The application manifest remains `0.1.0`; the prerelease tag describes distribution maturity. Publish it as a GitHub prerelease with source archives and a precise list of known limits. Do not attach the locally produced unsigned ZIP as a supported installer.

1. Review the complete source snapshot, license, notices, documentation links, and included images. Exclude generated runtime, reports, local profiles, backup folders, and private media.
2. Verify the selected commit with [Verify Tagryn](../.github/workflows/verify.yml). Record the commit and run URL. All four matrix jobs must pass before declaring the cross-platform check successful. A later source change needs a new run.
3. Confirm the chosen license before making a previously private source repository public. Enable Issues, Discussions, and private vulnerability reporting; verify their links from the public repository.
4. Create the prerelease against the verified commit with the prepared [release notes](releases/v0.1.0-alpha.1.md). Check the tag target, source downloads, prerelease flag, and visible limitations.

Full test runs belong in GitHub Actions. Existing historical local checks are not evidence for a later commit. Documentation-only corrections should be identified as such when recording verification.

## Supported installers

Before promoting binaries, follow the [runtime and distribution requirements](ARCHITECTURE.md#runtime-und-distribution). Sign and notarize all nested macOS executables correctly, sign Windows executables and installers, generate hashes, retain dependency notices, and test clean-machine installation plus read/save/restore on each supported platform.

Publish only artifacts built from the intended release commit. Signing keys stay in appropriately scoped secrets; the pull-request workflow has read-only repository permissions and does not receive signing secrets. CI artifacts are temporary development builds, not automatically published releases. There is no automatic updater or automatic release publication in this repository.
