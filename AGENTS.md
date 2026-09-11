# Working on Tagryn

Read [CONTRIBUTING.md](CONTRIBUTING.md) before editing. It defines contribution, verification, and review rules for humans and agents.

## Workflow

- Work on a focused branch in a new worktree. The integration branch is `develop`.
- Ask the maintainer for permission before any merge. Never commit, push, or merge into `main`.
- Write implementation plans under `.claude/plans/`. Include three viable approaches with tradeoffs when possible, and unresolved questions at the end.
- Run the complete test suite only through GitHub Actions. Locally, run only tests relevant to the change, including inside Docker.
- Report commands actually run, their results, and unverified behavior. A configured workflow is not a passing run.

## Read for the change

- Product scope or new features: [PRINCIPLES.md](docs/PRINCIPLES.md) and [ROADMAP.md](docs/ROADMAP.md).
- File writes, backup, restore, privacy, paths, process execution, or caches: [SAFETY.md](docs/SAFETY.md) and [ARCHITECTURE.md](docs/ARCHITECTURE.md).
- Tags, supported formats, RAW/XMP, or synchronization: [FORMATS.md](docs/FORMATS.md).
- Bundling, dependencies, or runtime updates: the distribution section of [ARCHITECTURE.md](docs/ARCHITECTURE.md), [THIRD-PARTY-NOTICES.md](docs/THIRD-PARTY-NOTICES.md), and [RELEASING.md](docs/RELEASING.md).

## Data integrity

Keep the flow explicit: draft → validated plan → verified backup → write → reread. Rust owns filesystem authorization and write validation. Preserve source/group/tag identity, fingerprint conflict checks, and independent per-file results. A preview failure must not become a metadata-read failure.

Use synthetic fixtures and disposable copies for write tests. Never alter the user's original media or publish private metadata, backup folders, profiles, or local reports. Changes to these boundaries require a focused regression test and an explanation of the safety effect.

Preserve offline operation and the bundled ExifTool runtime. Read scripts and lockfiles for actual build commands and versions. AI-generated code receives the same review as any other contribution.
