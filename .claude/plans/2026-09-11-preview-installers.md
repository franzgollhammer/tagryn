# Permanent preview installers

## Objective

Publish downloadable installers for macOS, Windows, and Linux on the existing `v0.1.0-alpha.1` GitHub release. Update both READMEs with direct links and installation instructions. Keep the application's tested source commit unchanged.

## Three approaches

1. **Package the verified release artifacts (selected).** Reuse the four successful native builds of the released commit, seal the macOS app with an ad-hoc signature, create a DMG for Apple Silicon and an app ZIP for Intel, and verify installation/runtime behavior on fresh GitHub runners. Publish permanent preview assets with checksums and provenance. This provides downloads now; Apple notarization and Windows publisher verification remain unavailable.
2. **Build a new binary release from current development.** Run the complete application matrix again and publish a new tag. This is useful when application code changes, but rebuilding unchanged code does not address the present distribution gap efficiently.
3. **Wait for distribution certificates.** Obtain Apple Developer ID/notarization and Windows signing credentials first. This gives fewer operating-system prompts but depends on credentials not currently configured. It does not meet the request for downloads now.

## Authorization and scope

The user explicitly requested downloads/installers for all three platforms after the source-only release. Publish clearly labeled alpha installers under that authorization. Earlier documentation describing source-only distribution is superseded by this request. Do not describe preview packages as signed/notarized production releases. The user's earlier merge approval applied to PR #1; obtain fresh permission before merging this change into `develop`.

## Diagnosis

The original macOS CI artifact repeatedly fails `codesign --verify --deep --strict` with `code has no resources but signature indicates they must be present`. Ranked hypotheses: incomplete bundle signing, resources changed after signing, or extraction damage. Archive executable modes are intact. The existing ad-hoc signing/check script succeeds on an isolated copy, including 53 Mach-O files and bundled ExifTool. The selected fix is to seal the complete app and verify it again after DMG creation, mounting, and copying to an installation directory.

## Work

1. Check release tag, successful source run, artifact identities, signing configuration, and primary installation documentation.
2. Add a dedicated packaging workflow using read-only repository/Actions access. Verify the source run matches the immutable release tag and completed successfully before using its artifacts.
3. Create an Apple Silicon DMG and an Intel app ZIP, preserve the original Windows NSIS and Linux Debian installer bytes, and test the installed private runtime with synthetic read/write/restore operations. Check installed app startup where runner environments allow it.
4. Verify hashes, package contents, licenses, OS architecture, and the final downloadable files. Keep signatures and installation-test limits explicit.
5. Upload permanent installer assets, checksums, and build provenance to the existing prerelease. Update its notes and verify public downloads without a GitHub login.
6. Prepare and validate English/German README changes and the packaging process documentation. Open a reviewable PR; ask for merge permission after the downloads are live.

## Verification strategy

The complete application suite has passed on all four native builds of commit `9998760f296047fb1335dee08533416aa9b4eb14`. Packaging uses those exact artifacts and checks their provenance. Dedicated native packaging checks run in GitHub Actions; focused local checks cover the macOS DMG loop and script syntax. Full local suites are prohibited. Packaging-only and documentation-only paths are separated from the application matrix so unchanged application code is not rebuilt for this work.

## Packaging findings

- macOS startup must use a canonical temporary path because Tauri rejects symlinks in executable paths. The isolated acceptance launch uses `-ApplePersistenceIgnoreState YES` to avoid restoring crash dialogs from previous runner sessions.
- The Intel Perl artifact included optional DBM bindings linked against Homebrew libraries. Packaging omits those unused bindings after checking that ExifTool does not reference them; strict native dependency and signature verification remain enabled.
- Linux's original headless probe produced a fully rendered app and a valid profile but no frontend readiness marker. Running a window manager within Xvfb and capturing both output streams made the original installed-app readiness check pass.
- Intel disk image failures narrowed to a repeatable `timeout for DiskArbitration expired` during eject. File-backed output and forced eject attempts did not fix it. The Intel package therefore uses a standard `ditto` application ZIP, followed by extraction, strict signature verification, and native runtime/startup checks. Apple Silicon retains its verified DMG. Both contain compiled applications; GitHub's separate source ZIP remains a source archive. The failed eject workaround was removed.
- Metadata probes touch only generated media in disposable directories. The probe verifies the installed private runtime and exact fixture restoration; application-level write-plan, backup, and restore behavior remains covered by the complete source CI suite.

## Final native verification

[Packaging run 34609674849](https://github.com/franzgollhammer/tagryn/actions/runs/34609674849) passed all four jobs at packaging commit `64e15e91fa0efa99269e43e6e92901184429754b`. Native checks installed the Windows EXE and Linux DEB, mounted/copied the Apple Silicon DMG, and extracted the Intel app ZIP. All verified license notices, ExifTool 13.59 with system PATH disabled, Unicode metadata read/write, exact fixture backup restoration, isolated profile creation, frontend readiness, and continued app startup. Both macOS installed bundles passed strict signature verification. Application source, manifests, and lockfiles remain unchanged from the release commit.

Focused local validation covered script syntax, documentation formatting, 55 relative links/anchors, and a DMG creation/compression/verification probe. Full application suites were not run locally; their successful release-commit GitHub Actions run is linked above.

## Publication

The existing `v0.1.0-alpha.1` prerelease now contains four compiled downloads, `BUILD-INFO.json`, and `SHA256SUMS.txt`. All six were downloaded through unauthenticated public URLs, returned HTTP 200, and matched their locally verified SHA-256 hashes. GitHub's asset digests also match. The release remains a prerelease and its tag still points to `9998760f296047fb1335dee08533416aa9b4eb14`. Release notes describe the actual formats, native checks, ad-hoc/unsigned status, and alpha limits. The final branch commit changes documentation only; packaging verification applies to its unchanged parent implementation.

## Unresolved questions

- May the finished packaging/README PR be merged into `develop`? Ask after validation and publication.
- Which Apple Developer ID and Windows signing identities should a future trusted installer release use? This does not block the explicitly requested alpha downloads.
- Should future Linux releases add AppImage, RPM, or ARM packages? The current verified Linux build is Ubuntu/Debian x64.
