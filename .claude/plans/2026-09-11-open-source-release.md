# Tagryn open source release

## Objective

Publish Tagryn under `franzgollhammer/tagryn` as an open source project that welcomes contributions made with AI. Omarchy is a reference for a clear product direction, accessible onboarding, explicit agent guidance, and maintainer-led review. This does not imply a relationship with Omarchy.

## Three release approaches

1. **Public source alpha (recommended).** Publish the working source, an open source license, contributor documentation, issue forms, and full GitHub Actions verification. This gives contributors a useful starting point quickly. The cost is requiring a development toolchain; signed installers remain a later milestone.
2. **Binary preview launch.** Complete the operating-system matrix, signing, notarization, clean-machine installation checks, and versioned installers before launch. This lowers the barrier for end users but depends on signing credentials and more platform testing.
3. **Website-led launch.** Add a separate product site, demo video, public roadmap, and installer release together. This gives stronger product communication but increases scope and delays community access.

## Selected implementation

Use the source-alpha path. The existing app remains version 0.1.0; the first public prerelease is identified as `v0.1.0-alpha.1`. MIT is proposed because it matches the reference project and permits broad reuse. Apache-2.0 and GPL-3.0 were offered as alternatives. An unanswered license preference is not approval to publish a particular license; prepare the candidate and obtain a decision before public licensing.

Keep development in a new worktree at `~/dev/worktrees/tagryn/feat-open-source-release/`. The original `feat/desktop-app` worktree contains uncommitted work and remains untouched. The repository has no history or remote. Bootstrap a `develop` default branch from the reviewed initial source snapshot; do not create, commit to, push to, or merge into `main`. No merge is required for an initial repository.

## Work

1. Inspect the local application, workflow, dependencies, generated material, and original validation notes. Research the supplied Omarchy pages using primary sources and record the comparison.
2. Prepare an English README with a German usage guide, product principles, contributor guidance for humans and agents, a compact AGENTS.md, issue forms, PR template, security-reporting instructions, and a practical roadmap.
3. Add consistent project license metadata while retaining third-party license terms. Exclude generated runtime, local reports, fixture media, private profiles, and unsigned local packages from publication. Review each promotional image before inclusion.
4. Verify all documentation links, metadata, workflow references, whitespace, and the staged source snapshot. Run the complete test suite only through GitHub Actions. Use a private repository for prepublication CI if needed.
5. Once licensing is decided and the public snapshot is concrete, publish the repository and a source prerelease with honest verification status and known limitations. Enable Issues, Discussions, and private vulnerability reporting. Verify the public repository, license detection, default branch, release target, and workflow outcome.

## Completion criteria

The public repository and source prerelease exist under the requested account, the license is chosen, contributor entry points are functional, and validation results are reported accurately. Production installer readiness is not claimed. If public licensing remains undecided, preserve a reviewable candidate and explicitly request that final decision.

## Initial CI findings and correction

The first full GitHub Actions run failed. On macOS/Linux, `corrupt_and_readonly_files_never_get_written` expected a read error for plain text named `corrupt.jpg`. A focused local rerun reproduced the assertion. Three hypotheses were considered: the fixture was valid text; the engine swallowed an ExifTool error; or the service returned cached data. Direct bundled-ExifTool probes showed that plain text is correctly recognized as TXT with exit status 0, while binary invalid input reports `File format error` with exit status 1. Replacing the fixture with genuinely unrecognized binary content made the existing test pass; the product error handling was unchanged. The generated review fixture now uses the same invalid bytes.

Windows integration-test executables failed before the test harness with `0xc0000139 (STATUS_ENTRYPOINT_NOT_FOUND)`. Ranked causes were a missing Common Controls v6 manifest, a conflicting DLL on PATH, or a missing C++ runtime. Tauri's own documented Windows test failure matches the first cause: https://github.com/tauri-apps/tauri/issues/11028 . The locked `embed-resource` implementation emits the app resource only for binary targets, excluding integration tests and examples. Add the Common Controls dependency through MSVC linker arguments scoped to those development executables, including the CI smoke example, without changing production manifests. The Windows CI rerun is the verification; a local macOS pass cannot establish a Windows fix.

## Windows Unicode correction

The manifest rerun verified all macOS/Linux jobs and the Windows test loader. Windows then reproduced a real write-verification failure for Unicode titles containing a line break. The old engine moved such values from UTF-8 stdin to the launcher's narrow argv, which uses the system codepage. A separate short Windows Actions probe compared the same values: argv changed `Grüße` to `Gr??e`, while UTF-8 argument input plus ExifTool `-ec` preserved all tested bytes, including Japanese text, emoji, shell-like literals, quotes, backslashes, and Unicode filenames. The passed probe is https://github.com/franzgollhammer/tagryn/actions/runs/34597250360 .

Three mechanisms were considered: argv codepage conversion, newline normalization, and command-line parsing. The value comparisons support codepage conversion. Three transport approaches were evaluated: keeping argv (incorrect on Windows), `#[CSTR]` argument lines (the bundled ExifTool changes dollar/at-sign literals), and UTF-8 argument lines with byte-escaped write values through `-ec` (selected). Unix argv remains available for filenames containing actual line breaks. Windows isolated reads also use UTF-8 stdin. No value files or additional unencrypted temporary metadata files are introduced.

A new service-level regression first failed for leading spaces and now checks exact write/read/isolated-read/restore behavior for whitespace, literal escape sequences, Unicode, and line breaks. Each iteration uses a distinct job ID because backup directories are intentionally not overwritten. The original multiline save/restore regression remains part of verification. The complete suite must be rerun on the resulting commit through GitHub Actions.

## Unresolved questions

- Which license should apply: MIT (recommended), Apache-2.0, or GPL-3.0?
- Which signing identities will be used for later macOS and Windows installer releases? This does not block the source alpha.
- Is a separate Tagryn domain and website desired after the repository launch? This does not block the source alpha.
