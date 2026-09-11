---
title: 'Introducing Tagryn: an offline metadata workbench built with Tauri and Rust'
published: false
description: 'Inspect, compare, and deliberately edit file metadata with Tagryn, an open source desktop app powered by Tauri, Rust, Vue, and ExifTool.'
tags: opensource, rust, tauri, vue
cover_image: https://raw.githubusercontent.com/franzgollhammer/tagryn/6efec1a308fba1541dbbea8265903298d6800a48/docs/images/tagryn-devto-cover.png
---

Two copies of a photo can look identical while telling different stories in their metadata. A timestamp differs. An author appears in one metadata group and is missing from another. A neighboring XMP sidecar contains yet another value.

Before changing anything, you need to understand where those values came from—and exactly what saving an edit will do.

I'm sharing [Tagryn](https://github.com/franzgollhammer/tagryn), an open source desktop workbench for inspecting, comparing, and deliberately editing file metadata. It uses Tauri, Rust, Vue, and ExifTool, and its core workflows run offline. No account, no telemetry, no automatic map requests.

I'm writing this to show what you can try today and explain the decisions behind the editing workflow. If you work with media files, Tagryn gives you a way to inspect their metadata and review changes before saving. If you're building a desktop app that modifies user files, the validation, backup, and verification flow may be useful in your own work.

![Tagryn's light-themed desktop workspace showing generated test images](https://raw.githubusercontent.com/franzgollhammer/tagryn/6efec1a308fba1541dbbea8265903298d6800a48/docs/images/workspace-light.png)

_The native macOS app with synthetic images. The interface supports English and German._

You can open files or folders, scan recursively, browse a list or thumbnail grid, and search metadata that has been loaded. Select up to eight files to compare their metadata, including a view that shows only differences.

For editing, Tagryn supports selected fields such as titles, descriptions, keywords, creators, ratings, capture times, and GPS values. Batch operations, presets, metadata transfers, renaming, and mapped CSV/JSON imports use the same reviewable planning workflow.

For example, imagine a folder of JPEGs that need consistent creator credits and keywords. Open disposable copies, inspect the existing values, compare a few files, then stage your changes across the selection. Review the plan before saving and inspect the result for each file afterward.

One detail matters here: **XMP is the default write target.** Synchronizing supported fields into EXIF/IPTC is an explicit option for JPEG and TIFF. A single “author” input should make its write targets clear. The [format documentation](https://github.com/franzgollhammer/tagryn/blob/develop/docs/FORMATS.md) explains those rules and their limits.

The save workflow is central to the app:

1. **Stage and validate.** The interface collects drafts. Rust checks the actual files, fingerprints, supported fields, values, and destinations to produce a plan.
2. **Prepare a candidate.** ExifTool applies changes to a temporary copy. Tagryn reads that candidate back and checks the planned operations.
3. **Back up and replace.** Before replacing the original, Tagryn creates and verifies a backup and checks again for conflicts.
4. **Read the saved result.** The app rereads the file and records success, warnings, errors, or skipped work separately for each file.

A batch can partially succeed. Restoration also checks the recorded backup and saved-file hashes; later edits by another program can prevent automatic restoration. These behaviors are documented in the [write and restore protocol](https://github.com/franzgollhammer/tagryn/blob/develop/docs/SAFETY.md).

For developers, the most interesting part may be the boundary between the interface and file operations.

Vue and Pinia manage selection, presentation, and unsaved drafts. Controlled Tauri commands connect them to a Rust service, which owns filesystem authorization, validation, planning, and write jobs. ExifTool handles metadata processing. SQLite stores caches, search data, settings, and job history; files and their sidecars remain the source of truth.

The metadata model preserves more than a tag's display name. Source, group, instance, and tag identity distinguish values that would collide in a simple name-to-value map. Embedded metadata and sidecar metadata retain their provenance, so conflicting values remain visible. The [architecture notes](https://github.com/franzgollhammer/tagryn/blob/develop/docs/ARCHITECTURE.md) go into the implementation.

ExifTool and its runtime ship with the desktop packages. You don't need Node.js, Rust, or a separate Perl installation to use a downloaded build. Building from source requires development tools and an initial download step; the app's core workflows then work offline.

**Tagryn is currently an alpha.** The [v0.1.0-alpha.1 release](https://github.com/franzgollhammer/tagryn/releases/tag/v0.1.0-alpha.1) includes preview downloads for macOS Apple Silicon and Intel, Windows x64, and Ubuntu or compatible Debian-based Linux x64. macOS packages are ad-hoc signed without Apple notarization; Windows installers are unsigned. Installation instructions, checksums, and build provenance accompany the release.

Read and write support differ by format. RAW edits use XMP sidecars. Embedded audio, video, and PDF editing is not implemented. Neither are GPX matching, online maps, live filesystem watching, or automatic updates.

Start with disposable copies and keep independent backups. Tagryn performs real writes and does not promise complete anonymization: original metadata can remain in sidecars, backups, caches, history, or exports. Removing metadata from a media file does not clean every local copy of that information.

Tagryn's own code and documentation use the MIT license. Contributions are welcome, including AI-assisted work. Contributors remain responsible for understanding their changes, verifying behavior, and responding to review. Useful starting points include English translations of the technical docs, accessibility checks, synthetic format fixtures, and focused bug fixes. See the [contribution guide](https://github.com/franzgollhammer/tagryn/blob/develop/CONTRIBUTING.md) for setup and verification rules.

Try the [desktop alpha](https://github.com/franzgollhammer/tagryn/releases/tag/v0.1.0-alpha.1) or explore the [source](https://github.com/franzgollhammer/tagryn).

Which metadata task takes more effort than it should? What would you want to know before trusting an app to edit your files?

Leave your questions in the comments—about Tagryn, the Tauri/Rust stack, or how the save workflow handles failures. Specific examples of metadata tasks you struggle with would be especially useful.
