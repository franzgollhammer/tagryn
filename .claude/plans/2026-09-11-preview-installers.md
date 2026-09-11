# Permanent preview installers

## Objective

Publish downloadable installers for macOS, Windows, and Linux on the existing `v0.1.0-alpha.1` GitHub release. Update both READMEs with direct links and installation instructions. Keep the application's tested source commit unchanged.

## Three approaches

1. **Package the verified release artifacts (selected).** Reuse the four successful native builds of the released commit, seal the macOS app with an ad-hoc signature, create DMGs, and verify installation/runtime behavior on fresh GitHub runners. Publish permanent preview assets with checksums and provenance. This provides downloads now; Apple notarization and Windows publisher verification remain unavailable.
2. **Build a new binary release from current development.** Run the complete application matrix again and publish a new tag. This is useful when application code changes, but rebuilding unchanged code does not address the present distribution gap efficiently.
3. **Wait for distribution certificates.** Obtain Apple Developer ID/notarization and Windows signing credentials first. This gives fewer operating-system prompts but depends on credentials not currently configured. It does not meet the request for downloads now.

## Authorization and scope

The user explicitly requested downloads/installers for all three platforms after the source-only release. Publish clearly labeled alpha installers under that authorization. Earlier documentation describing source-only distribution is superseded by this request. Do not describe preview packages as signed/notarized production releases. The user's earlier merge approval applied to PR #1; obtain fresh permission before merging this change into `develop`.

## Diagnosis

The original macOS CI artifact repeatedly fails `codesign --verify --deep --strict` with `code has no resources but signature indicates they must be present`. Ranked hypotheses: incomplete bundle signing, resources changed after signing, or extraction damage. Archive executable modes are intact. The existing ad-hoc signing/check script succeeds on an isolated copy, including 53 Mach-O files and bundled ExifTool. The selected fix is to seal the complete app and verify it again after DMG creation, mounting, and copying to an installation directory.

## Work

1. Check release tag, successful source run, artifact identities, signing configuration, and primary installation documentation.
2. Add a dedicated packaging workflow using read-only repository/Actions access. Verify the source run matches the immutable release tag and completed successfully before using its artifacts.
3. Create separate Apple Silicon and Intel DMGs, preserve the original Windows NSIS and Linux Debian installer bytes, and test the installed private runtime with synthetic read/write/restore operations. Check installed app startup where runner environments allow it.
4. Verify hashes, package contents, licenses, OS architecture, and the final downloadable files. Keep signatures and installation-test limits explicit.
5. Upload permanent installer assets, checksums, and build provenance to the existing prerelease. Update its notes and verify public downloads without a GitHub login.
6. Prepare and validate English/German README changes and the packaging process documentation. Open a reviewable PR; ask for merge permission after the downloads are live.

## Verification strategy

The complete application suite has passed on all four native builds of commit `9998760f296047fb1335dee08533416aa9b4eb14`. Packaging uses those exact artifacts and checks their provenance. Dedicated native packaging checks run in GitHub Actions; focused local checks cover the macOS DMG loop and script syntax. Full local suites are prohibited. Packaging-only and documentation-only paths are separated from the application matrix so unchanged application code is not rebuilt for this work.

## Unresolved questions

- May the finished packaging/README PR be merged into `develop`? Ask after validation and publication.
- Which Apple Developer ID and Windows signing identities should a future trusted installer release use? This does not block the explicitly requested alpha downloads.
- Should future Linux releases add AppImage, RPM, or ARM packages? The current verified Linux build is Ubuntu/Debian x64.
