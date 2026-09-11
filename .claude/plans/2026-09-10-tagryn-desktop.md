# Tagryn desktop implementation

## Product direction

Tagryn is an offline metadata workbench for photographers and editorial teams. The default workspace has a folder sidebar, a virtualized file browser, and a metadata inspector. A persistent change tray separates planning, saving, discarding unsaved edits, and restoring saved files. Jade accents indicate selection and primary actions; amber indicates pending changes.

## Three implementation approaches

1. **A Rust application service with an embedded ExifTool runtime (selected).** Vue and Pinia manage presentation and drafts. Rust owns file capabilities, metadata identities, fingerprints, plans, backups, jobs, and SQLite. This gives explicit safety boundaries and small UI overhead. The cost is maintaining the runtime on each operating system.
2. **A Rust service spawning ExifTool for each file.** This is easier to debug and isolates failures, but process startup becomes expensive in large folders. Retain a bounded one-shot path only for binary previews and exceptional operations.
3. **A separate long-running metadata service with a local RPC socket.** This could serve multiple windows and other clients, but introduces authentication, deployment, and lifecycle complexity. It is unnecessary for the initial desktop app.

## Names and identity

| Name | Rationale | Preliminary collision check, 2026-09-10 |
| --- | --- | --- |
| **Tagryn** | A compact invented name combining tags with a suggestion of grain. | Selected. Search found an existing personal Flickr handle, but no obvious metadata application. Domain and trademark availability are unverified. |
| Lumeta | Light plus metadata; easy to say. | Existing AI and network security software brands. Rejected. |
| Framio | Image frames and an approachable tool name. | Existing EXIF framing application and several other apps. Rejected. |
| Metra | Precision and measurement. | Established transport brand and associated apps. Rejected. |
| Picta | Direct photographic association. | Existing photo printing and photo editing products. Rejected. |

Sources: https://www.flickr.com/photos/tagryn/34012233040/ , https://www.lumeta.fun/ , https://framio.org/ , https://metra.com/ , https://picta.com/app/ . This is an obvious-collision screen, not legal clearance.

Three editable vector studies: (A) an open frame with a displaced jade detail, (B) three offset metadata layers, (C) a divided image and detail window. Select A for its strong small-size silhouette. Deliver separate compact optical master, SVG, PNG sizes, ICNS and ICO, a light/dark specimen sheet, and actual bundle integration.

## Architecture and boundaries

- `src/domain`: types, field definitions, comparison and import validation.
- `src/stores`: workspace selection, preferences, and pending edits.
- `src/components`: accessible desktop controls, inspector, virtualized browser, workflow dialogs.
- `src-tauri/src/engine`: bounded persistent ExifTool workers, unique request IDs, timeout and restart, raw/formatted values, disabled user configuration.
- `src-tauri/src/files`: canonical paths, explicit opened-file registry, fingerprints, preview and directory enumeration.
- `src-tauri/src/planner`: allowlisted operations, exact diffs and sidecar policy.
- `src-tauri/src/jobs`: sequential commit per file, safe cancellation, verified backups and independent results.
- `src-tauri/src/db`: bounded metadata cache, preferences, presets and durable job journal.

Writes are prepared on a sibling temporary file. The original is fingerprinted again before replacement; a backup is created and verified first. Each result is reread. No batch-wide atomicity is promised. RAW edits default to XMP sidecars. EXIF/IPTC/XMP synchronization is selected explicitly. Sidecar conflicts remain visible. No automatic network requests occur at runtime.

## Delivery sequence

1. Scaffold the isolated worktree and lock verified dependencies; build the vendored ExifTool and Perl runtime.
2. Implement file opening, exact metadata identities, one allowlisted XMP edit, backup, reread and restore.
3. Extend selection, diff/compare, privacy plans, presets, transfer, dates/GPS, sidecars, rename, import/export, jobs and persistence.
4. Complete the desktop interaction design, keyboard shortcuts, German/English, themes, resizable panes and icons.
5. Compile and run focused integrity checks; configure the complete test suite exclusively in GitHub Actions. Measure native startup/RSS and file scans, inspect both themes and sizes, then build and inspect the macOS bundle.

## Validation requirements

Cover duplicate tags, Unicode and hostile filenames/values, malformed files, read-only files, external modification, sidecar conflicts, backup integrity, cancellation and partially failed batches. Never run the full suite locally. Use generated disposable fixtures for destructive checks. Never write to the user's existing photographs. Windows/Linux release verification requires the corresponding CI runners; unsigned local macOS builds are not equivalent to signed release packages.

## Unresolved questions

### Native timing harness diagnosis

The product's native edit/save/restore path passed, but `node scripts/measure-native.mjs` produced a startup timeout and an unreliable accelerator-driven shutdown. The minimal reproduction uses one real JPEG and one fresh profile. Three candidate explanations are ranked: (1) background-window throttling of the requestAnimationFrame readiness signal, predicted to disappear on explicit activation; (2) overlap with a previous native process, predicted to disappear in a single isolated run; (3) signed-runtime failure, predicted to reproduce in a direct metadata read from the signed bundle. Test one variable at a time. The existing native harness is the regression seam; no artificial mock of the renderer is needed.

The isolated background run still timed out, while a direct read through the signed bundled runtime succeeded. Activating the same single-file run made it pass at 1,306 ms with a normal close. This confirms the visibility-sensitive readiness marker as the timing-harness issue, not an engine regression. The harness now activates the specific application PID and closes the native window explicitly. The original three-run scenario is the final regression check. `--background` retains the minimal diagnostic reproduction; no temporary product instrumentation was introduced.

- Which GitHub repository should host the full Actions run? No remote exists yet; preparation and focused local validation can proceed.
- Distribution signing identities and notarization credentials have not been supplied. Local application builds can proceed unsigned/ad-hoc signed.
- Trademark and domain clearance remain outside the preliminary name check.
