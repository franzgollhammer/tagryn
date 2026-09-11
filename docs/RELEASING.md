# Releasing Tagryn

Releases are maintained by Franz Gollhammer. The integration branch is `develop`; `main` is not used. Obtain explicit maintainer permission before any merge.

## Source alpha

The initial source-only release was published as `v0.1.0-alpha.1`; it now also carries the preview installers described below. The application manifest remains `0.1.0`; the prerelease tag describes distribution maturity. Publish it as a GitHub prerelease with source archives and a precise list of known limits. Do not attach the locally produced unsigned ZIP as a supported installer.

1. Review the complete source snapshot, license, notices, documentation links, and included images. Exclude generated runtime, reports, local profiles, backup folders, and private media.
2. Verify the selected commit with [Verify Tagryn](../.github/workflows/verify.yml). Record the commit and run URL. All four matrix jobs must pass before declaring the cross-platform check successful. A later source change needs a new run.
3. Confirm the chosen license before making a previously private source repository public. Enable Issues, Discussions, and private vulnerability reporting; verify their links from the public repository.
4. Create the prerelease against the verified commit with the prepared [release notes](releases/v0.1.0-alpha.1.md). Check the tag target, source downloads, prerelease flag, and visible limitations.

Full test runs belong in GitHub Actions. Existing historical local checks are not evidence for a later commit. Documentation-only corrections should be identified as such when recording verification.

## Preview installers

The maintainer requested permanent desktop downloads for `v0.1.0-alpha.1` after its initial source-only publication. That prerelease may therefore include explicitly labeled preview installers: ad-hoc signed macOS apps in an Apple Silicon DMG and an Intel app ZIP, an unsigned Windows NSIS installer, and an unsigned Linux Debian package. This does not establish production signing or notarization.

[Package preview installers](../.github/workflows/package-preview.yml) packages an existing successful Verify Tagryn run. Its source run must match the exact existing release tag. It verifies the downloaded artifact digest, checks installed license notices and runtime behavior, and requires the installed frontend to report readiness. macOS checks also seal the bundle, verify every native dependency/deployment target, mount/copy the Apple Silicon DMG or extract the Intel app ZIP, and check the installed signature again. The private macOS Perl runtime omits optional `DB_File`, `GDBM_File`, and `NDBM_File` bindings to avoid host Homebrew dependencies; a guard rejects ExifTool sources referencing those modules.

The Intel download uses an app ZIP because the native CI runner repeatedly timed out ejecting disk images. It contains the compiled application, preserves its signature, and receives the same native runtime/startup checks. Extracting it and moving `Tagryn.app` to Applications installs it.

Run it manually with a release tag and source run while the source artifacts are retained:

```sh
gh workflow run package-preview.yml --ref develop -f release_tag=v0.1.0-alpha.1 -f source_run=34597693265
```

After all four native packaging jobs pass, download their `preview-*` artifacts, verify the four installer hashes against their individual `build-info-*.json` records, and combine those records into `BUILD-INFO.json`. Generate `SHA256SUMS.txt` for the installers and provenance file. Upload only those reviewed files to the intended prerelease with `gh release upload`; never substitute an artifact from a failed run or replace published files without deliberately reviewing the replacement. Verify public downloads without authentication, their bytes, architecture labels, and release notes.

The packaging workflow has read-only repository/Actions permissions and does not publish releases or use distribution keys. Publishing remains a separate maintainer action. Application code changes still require the complete Verify Tagryn matrix; packaging-only changes run their own native installer checks against the tested application commit. Keep source verification, packaging verification, and signing claims separate in the release notes.

## Supported installers

Before promoting preview binaries to supported production installers, follow the [runtime and distribution requirements](ARCHITECTURE.md#runtime-und-distribution). Sign and notarize all nested macOS executables correctly, sign Windows executables and installers, generate hashes, retain dependency notices, and test clean-machine installation plus read/save/restore on each supported platform.

Publish only artifacts built from the intended release commit. Signing keys stay in appropriately scoped secrets; the pull-request workflow has read-only repository permissions and does not receive signing secrets. CI artifacts are temporary development builds, not automatically published releases. There is no automatic updater or automatic release publication in this repository.
