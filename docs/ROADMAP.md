# Roadmap

Tagryn starts as a source alpha. This roadmap describes priorities rather than promised dates. Discuss substantial work in an issue before starting it.

## Make the first release dependable

- Pass the complete GitHub Actions matrix for macOS Apple Silicon, macOS Intel, Windows x64, and Linux x64.
- Exercise install, launch, read, save, and restore on clean machines. Complete macOS signing/notarization and Windows signing before supported installer releases.
- Expand adversarial-file, disk-full, interrupted-write, and platform filesystem coverage.
- Publish release notes that distinguish tested behavior from known limits.

## Make contribution easy

- Translate the architecture, format rules, and safety documentation into English.
- Improve setup instructions from fresh contributor machines.
- Add minimized synthetic fixtures for reproducible format issues.
- Check keyboard navigation, screen readers, and layouts at larger text sizes.

## Explore after the foundations

- Broader camera and format coverage, backed by explicit read/write expectations.
- Better visibility and deliberate retention controls for local caches, backup data, and history.
- GPX matching, paired medium/sidecar rename, and filesystem watching after their conflict behavior is specified.

Cloud accounts, telemetry, and automatic online lookups are outside the current product direction. See [FORMATS.md](FORMATS.md) for implemented capabilities and [VALIDATION.md](VALIDATION.md) for the historical local checks.
